#!/usr/bin/env python3
"""Check local Markdown links and #anchors in docs and the bundled skill without network calls."""
import re
import unicodedata
from functools import cache
from pathlib import Path
from urllib.parse import unquote, urlsplit

root = Path(__file__).resolve().parent.parent
FENCE = re.compile(r'^(```|~~~).*?^\1[^\n]*$', flags=re.S | re.M)
HEADING = re.compile(r'^#{1,6}[ \t]+(.+?)[ \t#]*$', flags=re.M)
EXPLICIT_ID = re.compile(r'\s*\{#([^}\s]+)\}\s*$')
HTML_ID = re.compile(r'<[a-zA-Z][^>]*\s(?:id|name)=["\']([^"\']+)["\']')
SPECIAL = re.compile(r'[\s~`!@#$%^&*()\-_+=\[\]{}|\\;:"\'“”‘’<>,.?/]+')


def strip_code(text):
    return FENCE.sub('', text)


def slugify(text):
    """Match VitePress's default slugify (@mdit-vue/shared)."""
    text = unicodedata.normalize('NFKD', text)
    text = re.sub(r'[\u0300-\u036f]', '', text)
    text = re.sub(r'[\u0000-\u001f]', '', text)
    text = SPECIAL.sub('-', text)
    text = re.sub(r'-{2,}', '-', text).strip('-')
    text = re.sub(r'^(\d)', r'_\1', text)
    return text.lower()


def plain(text):
    """Reduce inline Markdown in a heading to its rendered text."""
    text = re.sub(r'!?\[([^\]]*)\]\([^)]*\)', r'\1', text)
    text = re.sub(r'<[^>]+>', '', text)
    text = re.sub(r'`([^`]*)`', r'\1', text)
    return re.sub(r'(\*\*|__|\*|_)(\S(?:.*?\S)?)\1', r'\2', text)


@cache
def anchors(path):
    text = strip_code(path.read_text())
    found, counts = set(HTML_ID.findall(text)), {}
    for heading in HEADING.findall(text):
        explicit = EXPLICIT_ID.search(heading)
        if explicit:
            found.add(explicit.group(1))
            continue
        slug = slugify(plain(heading))
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        found.add(slug if count == 0 else f'{slug}-{count}')
    return found


failures = []
for path in [root / 'README.md', root / 'RELEASING.md', *sorted((root / 'docs').rglob('*.md')), *sorted((root / 'skills').rglob('*.md'))]:
    if '.vitepress' in path.parts or 'node_modules' in path.parts or not path.exists():
        continue
    text = strip_code(path.read_text())
    for link in re.findall(r'\]\(([^)\s]+)(?:\s+"[^"]*")?\)', text):
        url = urlsplit(link.strip('<>'))
        if url.scheme or url.netloc:
            continue
        if url.path:
            name = unquote(url.path)
            target = root / 'docs' / name.lstrip('/') if name.startswith('/') else path.parent / name
            options = [target, Path(str(target) + '.md'), target / 'index.md']
            if name.startswith('/'):
                options.append(root / 'docs/public' / name.lstrip('/'))
            existing = next((option for option in options if option.exists()), None)
            if existing is None:
                failures.append(f'{path.relative_to(root)}: {link}')
                continue
        else:
            existing = path
        fragment = unquote(url.fragment)
        if fragment and existing.suffix == '.md' and fragment not in anchors(existing):
            failures.append(f'{path.relative_to(root)}: {link} (missing anchor #{fragment})')
if failures:
    raise SystemExit('\n'.join(failures))
print('Local Markdown links and anchors resolve')
