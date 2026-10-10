#!/usr/bin/env python3
"""Build a resumable Open Library holdout from works' librarian edition data."""
import json, pathlib, time, urllib.parse, urllib.request
ROOT = pathlib.Path(__file__).resolve().parents[1]
CACHE = ROOT / 'cache' / 'openlibrary'
CACHE.mkdir(parents=True, exist_ok=True)
STATE = CACHE / 'state.json'
OUT = ROOT / 'holdout.jsonl'
NS = 'https://openlibrary.org'

def get(path, state):
    delay = 1.05 - (time.time() - state['last'])
    if delay > 0: time.sleep(delay)
    req = urllib.request.Request(NS + path, headers={'User-Agent': 'PaperCorpusResearch/1.0 (Open Library API metadata corpus)'})
    with urllib.request.urlopen(req, timeout=60) as response: blob = response.read()
    state['last'] = time.time()
    STATE.write_text(json.dumps(state, sort_keys=True) + '\n')
    return blob

def save(state):
    STATE.write_text(json.dumps(state, sort_keys=True) + '\n')
    with OUT.open('w') as f:
        for row in state['rows']: f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + '\n')

def main():
    state = json.loads(STATE.read_text()) if STATE.exists() else {'last': 0.0, 'offset': 0, 'seen': [], 'rows': []}
    seen = set(state['seen']) | {row.get('source', {}).get('work') for row in state['rows']}
    seen.discard(None)
    state['seen'] = sorted(seen)
    state['rows'] = list({row['id']: row for row in state['rows']}.values())
    while len(state['rows']) < 500:
        q = urllib.parse.urlencode({'q':'lccn:*', 'limit':100, 'offset':state['offset'], 'fields':'key,title,edition_key,lccn'})
        blob = get('/search.json?' + q, state)
        docs = json.loads(blob).get('docs', [])
        if not docs: raise RuntimeError('Open Library search returned no more candidate works')
        state['offset'] += len(docs)
        save(state)
        for doc in docs:
            work = doc.get('key', '').rsplit('/', 1)[-1]
            if not work or work in seen: continue
            seen.add(work); state['seen'] = sorted(seen)
            path = f'/works/{work}/editions.json?limit=100'
            blob = get(path, state)
            entries = json.loads(blob).get('entries', [])
            for edition in entries:
                classes = edition.get('lc_classifications') or []
                if not classes: continue
                edition_id = edition.get('key', '').rsplit('/', 1)[-1]
                if not edition_id: continue
                row = {
                    'id': f'openlibrary:{edition_id}',
                    'title': edition.get('title', ''),
                    'abstract_text': '',
                    'authors': [],
                    'lc_classifications': classes,
                    'license': 'CC0 1.0',
                    'source': {'kind':'openlibrary','url':NS + edition.get('key', ''),
                               'metadata_license':'CC0 1.0', 'label_source':'lc_classifications',
                               'work': work},
                }
                state['rows'].append(row)
                break
            save(state)
            if len(state['rows']) % 25 == 0:
                print(f"Open Library holdout: {len(state['rows'])}/500; search offset={state['offset']}", flush=True)
            if len(state['rows']) == 500: break
        save(state)
    print('completed Open Library holdout:', len(state['rows']))

if __name__ == '__main__': main()
