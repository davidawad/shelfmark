#!/usr/bin/env python3
"""Polite resumable arXiv OAI-PMH harvest; cache is intentionally ignored by git.

Usage: harvest_arxiv.py               full window harvest -> cache/arxiv-all.xml
       harvest_arxiv.py --pinned IDS  arXivOld records for pinned IDs only
"""
import http.client, json, pathlib, sys, time, urllib.parse, xml.etree.ElementTree as ET

BASE = 'https://oaipmh.arxiv.org/oai'
ROOT = pathlib.Path(__file__).resolve().parent
if ROOT.name == 'builder':
    ROOT = ROOT.parent
ARCHIVES = ['cs', 'math', 'physics', 'q-bio', 'q-fin', 'stat', 'econ', 'eess']
FROM, UNTIL = '2026-01-01', '2026-10-09'
# arXivRaw is the primary record, but its acm-class element mirrors msc-class.
# arXivOld carries the submitted ACM classes, so the builder merges both.
CACHES = {'arXivRaw': ROOT / 'cache' / 'arxiv', 'arXivOld': ROOT / 'cache' / 'arxiv-old'}
NS = {'o': 'http://www.openarchives.org/OAI/2.0/'}
CONNECTION = None
LAST_REQUEST = 0.0

def request(params):
    """GET one OAI response, keeping at least 3.15 s between all requests."""
    global CONNECTION, LAST_REQUEST
    path = '/oai?' + urllib.parse.urlencode(params)
    for attempt in range(2):
        delay = 3.15 - (time.time() - LAST_REQUEST)
        if delay > 0: time.sleep(delay)
        try:
            if CONNECTION is None:
                CONNECTION = http.client.HTTPSConnection('oaipmh.arxiv.org', timeout=90)
            CONNECTION.request('GET', path, headers={'User-Agent': 'paper-corpus-research/1.0'})
            response = CONNECTION.getresponse()
            blob = response.read()
            if response.status != 200:
                raise RuntimeError(f'OAI HTTP {response.status}: {blob[:500]!r}')
            return blob
        except (OSError, http.client.HTTPException):
            CONNECTION.close()
            CONNECTION = None
            if attempt:
                raise
        finally:
            LAST_REQUEST = time.time()

def oai_error(root):
    err = root.find('.//o:error', NS)
    return None if err is None else ET.tostring(err, encoding='unicode')

def harvest_set(archive, prefix='arXivRaw'):
    d = CACHES[prefix] / archive.replace(':', '_')
    d.mkdir(parents=True, exist_ok=True)
    state_path = d / 'state.json'
    if state_path.exists():
        state = json.loads(state_path.read_text())
    else:
        state = {'token': None, 'done': False, 'pages': 0}
    while not state['done']:
        params = {'verb': 'ListRecords'}
        if state['token']:
            params['resumptionToken'] = state['token']
        else:
            params.update(metadataPrefix=prefix, set=archive, **{'from': FROM, 'until': UNTIL})
        blob = request(params)
        root = ET.fromstring(blob)
        # leave progress untouched on an OAI error so a rerun retries this page
        if (err := oai_error(root)) is not None:
            raise RuntimeError(err)
        page_no = state['pages']
        (d / f'{page_no:06d}.xml').write_bytes(blob)
        token_el = root.find('.//o:resumptionToken', NS)
        token = (token_el.text or '').strip() if token_el is not None else ''
        state.update(token=token or None, done=not bool(token), pages=page_no + 1)
        state_path.write_text(json.dumps(state, sort_keys=True) + '\n')
        records = len(root.findall('.//o:ListRecords/o:record', NS))
        print(f'{prefix} {archive}: page {page_no + 1}, {records} records, next={bool(token)}', flush=True)
    return d

def bare_id(ident):
    return ident.removeprefix('oai:arXiv.org:')

def cached_records(dirs, wanted=None):
    """Yield (identifier, record) once per identifier from cached pages."""
    seen = set()
    for d in dirs:
        for p in sorted(d.glob('*.xml')):
            page = ET.parse(p).getroot()
            for rec in page.findall('.//o:record', NS):
                ident = rec.findtext('o:header/o:identifier', namespaces=NS)
                if not ident or ident in seen:
                    continue
                if wanted is not None and bare_id(ident) not in wanted:
                    continue
                seen.add(ident)
                yield ident, rec

def write_combined(out, records):
    # Materialize an XML fixture consumed by the Rust deterministic builder,
    # keeping only one parsed page in memory at a time.
    count = 0
    with out.open('w') as f:
        f.write('<OAI-PMH><ListRecords>\n')
        for _, rec in records:
            f.write(ET.tostring(rec, encoding='unicode'))
            f.write('\n')
            count += 1
        f.write('</ListRecords></OAI-PMH>\n')
    print(f'combined OAI records: {count} ({out})')

def harvest_pinned(ids_path, prefix='arXivOld'):
    """Re-harvest only pinned IDs: ListRecords windows first, GetRecord for the rest."""
    wanted = set(pathlib.Path(ids_path).read_text().split())
    dirs = [harvest_set(archive, prefix) for archive in ARCHIVES]
    found = {bare_id(ident) for ident, _ in cached_records(dirs, wanted)}
    d = CACHES[prefix] / 'get-record'
    d.mkdir(parents=True, exist_ok=True)
    missing = sorted(wanted - found)
    print(f'{prefix}: {len(found)} pinned IDs in windows, {len(missing)} via GetRecord', flush=True)
    for n, arxiv_id in enumerate(missing, 1):
        path = d / (arxiv_id.replace('/', '_') + '.xml')
        if path.exists():
            continue
        blob = request({'verb': 'GetRecord', 'metadataPrefix': prefix,
                        'identifier': f'oai:arXiv.org:{arxiv_id}'})
        if (err := oai_error(ET.fromstring(blob))) is not None:
            raise RuntimeError(f'GetRecord {arxiv_id}: {err}')
        path.write_bytes(blob)
        print(f'{prefix} GetRecord {n}/{len(missing)}: {arxiv_id}', flush=True)
    records = list(cached_records(dirs + [d], wanted))
    got = {bare_id(ident) for ident, _ in records}
    if got != wanted:
        raise RuntimeError(f'missing pinned records: {sorted(wanted - got)[:20]}')
    write_combined(ROOT / 'cache' / f'{prefix}-pinned.xml', records)

def main():
    if sys.argv[1:2] == ['--pinned']:
        return harvest_pinned(sys.argv[2])
    dirs = [harvest_set(archive, prefix) for prefix in CACHES for archive in ARCHIVES]
    # arXivRaw and arXivOld share OAI identifiers; keep one record of each format.
    records = (r for prefix in CACHES
               for r in cached_records([d for d in dirs if d.parent == CACHES[prefix]]))
    write_combined(ROOT / 'cache' / 'arxiv-all.xml', records)

if __name__ == '__main__': main()
