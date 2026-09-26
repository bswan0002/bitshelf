#!/usr/bin/env python3
"""Check local Markdown links in docs and the bundled skill without network calls."""
import re
from pathlib import Path
from urllib.parse import unquote, urlsplit

root = Path(__file__).resolve().parent.parent
failures = []
for path in [root / 'README.md', root / 'RELEASING.md', *sorted((root / 'docs').rglob('*.md')), *sorted((root / 'skills').rglob('*.md'))]:
    if '.vitepress' in path.parts:
        continue
    text = re.sub(r'```.*?```', '', path.read_text(), flags=re.S)
    for link in re.findall(r'\]\(([^)]+)\)', text):
        url = urlsplit(link.strip('<>'))
        if url.scheme or url.netloc or not url.path:
            continue
        name = unquote(url.path)
        target = root / 'docs' / name.lstrip('/') if name.startswith('/') else path.parent / name
        options = [target, Path(str(target) + '.md'), target / 'index.md']
        if name.startswith('/'):
            options.append(root / 'docs/public' / name.lstrip('/'))
        if not any(option.exists() for option in options):
            failures.append(f'{path.relative_to(root)}: {link}')
if failures:
    raise SystemExit('\n'.join(failures))
print('Local Markdown links resolve')
