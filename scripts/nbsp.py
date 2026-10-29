#!/usr/bin/env python3
"""Twarda spacja po jednoliterowych słowach (a, i, o, u, w, z) w polskich plikach Markdown – poza kodem."""
import re, sys
PATTERN = re.compile(r'(?<![\w\u00a0])([aiouwzAIOUWZ]) (?=\S)')
for path in sys.argv[1:]:
    out, fenced = [], False
    for line in open(path, encoding='utf-8').read().split('\n'):
        if line.lstrip().startswith('```'):
            fenced = not fenced
            out.append(line)
            continue
        if fenced:
            out.append(line)
            continue
        parts = line.split('`')
        parts = [PATTERN.sub('\\1\u00a0', p) if i % 2 == 0 else p for i, p in enumerate(parts)]
        out.append('`'.join(parts))
    open(path, 'w', encoding='utf-8').write('\n'.join(out))
