#!/usr/bin/env python3
"""Polite resumable arXiv OAI-PMH harvest; cache is intentionally ignored by git."""
import http.client, json, pathlib, time, urllib.parse, xml.etree.ElementTree as ET

BASE = 'https://oaipmh.arxiv.org/oai'
ROOT = pathlib.Path(__file__).resolve().parent
if ROOT.name == 'builder':
    ROOT = ROOT.parent
CACHE = ROOT / 'cache' / 'arxiv'
ARCHIVES = ['cs', 'math', 'physics', 'q-bio', 'q-fin', 'stat', 'econ', 'eess']
FROM, UNTIL, PREFIX = '2026-01-01', '2026-10-09', 'arXivRaw'
NS = {'o': 'http://www.openarchives.org/OAI/2.0/'}
CONNECTION = None

def request(params):
    global CONNECTION
    path = '/oai?' + urllib.parse.urlencode(params)
    for attempt in range(2):
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
            time.sleep(3.15)

def harvest_set(archive):
    d = CACHE / archive.replace(':', '_')
    d.mkdir(parents=True, exist_ok=True)
    state_path = d / 'state.json'
    if state_path.exists():
        state = json.loads(state_path.read_text())
    else:
        state = {'token': None, 'done': False, 'pages': 0, 'last_request': 0}
    while not state['done']:
        delay = 3.15 - (time.time() - state.get('last_request', 0))
        if delay > 0: time.sleep(delay)
        params = {'verb': 'ListRecords'}
        if state['token']:
            params['resumptionToken'] = state['token']
        else:
            params.update(metadataPrefix=PREFIX, set=archive, **{'from': FROM, 'until': UNTIL})
        blob = request(params)
        page_no = state['pages']
        (d / f'{page_no:06d}.xml').write_bytes(blob)
        root = ET.fromstring(blob)
        token_el = root.find('.//o:resumptionToken', NS)
        token = (token_el.text or '').strip() if token_el is not None else ''
        state.update(token=token or None, done=not bool(token), pages=page_no + 1, last_request=time.time())
        state_path.write_text(json.dumps(state, sort_keys=True) + '\n')
        records = len(root.findall('.//o:ListRecords/o:record', NS))
        print(f'{archive}: page {page_no + 1}, {records} records, next={bool(token)}', flush=True)
        # malformed or transient OAI error response; preserve progress and retry once after delay
        if root.find('.//o:error', NS) is not None:
            raise RuntimeError(ET.tostring(root.find('.//o:error', NS), encoding='unicode'))
    return d

def main():
    roots = []
    for archive in ARCHIVES:
        d = harvest_set(archive)
        roots.append(d)
    # Materialize an XML fixture consumed by the Rust deterministic builder,
    # keeping only one parsed page in memory at a time.
    out = ROOT / 'cache' / 'arxiv-all.xml'
    seen = set()
    count = 0
    with out.open('w') as f:
        f.write('<OAI-PMH><ListRecords>\n')
        for d in roots:
            for p in sorted(d.glob('*.xml')):
                page = ET.parse(p).getroot()
                for rec in page.findall('.//o:ListRecords/o:record', NS):
                    ident = rec.findtext('o:header/o:identifier', namespaces=NS)
                    if ident and ident not in seen:
                        seen.add(ident)
                        f.write(ET.tostring(rec, encoding='unicode'))
                        f.write('\n')
                        count += 1
        f.write('</ListRecords></OAI-PMH>\n')
    print(f'combined unique OAI records: {count} ({out})')

if __name__ == '__main__': main()
