#!/usr/bin/env python3
"""Repeatable, isolated index-free scale benchmark; never uses the user's store."""
import argparse
import json
import os
import platform
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--bs', default='target/release/bs')
parser.add_argument('--counts', default='1000,10000')
parser.add_argument('--repeats', type=int, default=3)
args = parser.parse_args()
bs = str(Path(args.bs).resolve())
report = {'environment': platform.platform(), 'binary': subprocess.check_output([bs, '--version'], text=True).strip(), 'body_bytes': 2048, 'repeats': args.repeats, 'results': []}

with tempfile.TemporaryDirectory(prefix='bs-benchmark-') as directory:
    base = Path(directory)
    config = base / 'config.toml'
    store = base / 'store'
    config.write_text(f'store = {json.dumps(str(store))}\n')
    for shelf in ['notes', 'archive', 'expiry']:
        (store / shelf / 'bits').mkdir(parents=True)
    (store / 'expiry' / 'bs.toml').write_text('retention = "1d"\n')
    body = ('Realistic reusable note, Rust API documentation and handoff context.\n' * 40)[:2048]
    body_file = base / 'body.txt'
    body_file.write_text(body)
    def timed(argv):
        start = time.perf_counter()
        child = subprocess.Popen([bs, '--config', str(config), *argv], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        _, status, usage = os.wait4(child.pid, 0)
        child.returncode = os.waitstatus_to_exitcode(status)
        error = child.stderr.read().decode()
        child.stderr.close()
        if child.returncode:
            raise RuntimeError(f'{argv}: {error}')
        rss_bytes = usage.ru_maxrss * (1 if platform.system() == 'Darwin' else 1024)
        return time.perf_counter() - start, rss_bytes / (1024 * 1024)
    previous = 0
    for count in map(int, args.counts.split(',')):
        for i in range(previous, count):
            (store / 'notes' / 'bits' / f'note-{i:05}.md').write_text(f'---\ntitle: Note {i}\ntags: [project:bitshelf, example]\ncustom: {{source: benchmark, count: {i}}}\n---\n{body}')
        previous = count
        cases = {
            'list': ['list', '--json'],
            'search': ['search', 'reusable Rust', '--json'],
            'add': ['add', 'notes/benchmark-add', '--file', str(body_file)],
            'edit': ['edit', 'notes/note-00000', '--title', 'Edited'],
            'move': ['move', 'notes/note-00001', 'archive'],
            'prune': ['prune', '--json'],
        }
        for name, command in cases.items():
            samples = []
            for repeat in range(args.repeats):
                if name == 'add':
                    (store / 'notes/bits/benchmark-add.md').unlink(missing_ok=True)
                if name == 'edit':
                    command[-1] = f'Edited {count}/{repeat}'
                if name == 'move' and (store / 'archive/bits/note-00001.md').exists():
                    (store / 'archive/bits/note-00001.md').rename(store / 'notes/bits/note-00001.md')
                if name == 'prune':
                    for i in range(10):
                        (store / 'expiry/bits' / f'expired-{i}.md').write_text("---\nexpires: '2000-01-01T00:00:00Z'\n---\nexpired")
                samples.append(timed(command))
            report['results'].append({'bits': count, 'operation': name, 'median_ms': round(statistics.median(s[0] for s in samples) * 1000, 2), 'peak_mib': round(max(s[1] for s in samples), 2)})
        start = time.perf_counter()
        peak = 0
        for i in range(100):
            _, rss = timed(['add', f'notes/import-{count}-{i}', '--file', str(body_file)])
            peak = max(peak, rss)
        report['results'].append({'bits': count, 'operation': '100 sequential imports', 'median_ms': round((time.perf_counter() - start) * 1000, 2), 'peak_mib': round(peak, 2)})
print(json.dumps(report, indent=2))
