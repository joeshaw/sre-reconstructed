#!/usr/bin/env python3
"""Render a captured CP437 ANSI byte stream as plain text (transcript)."""
import re, sys
data = open(sys.argv[1], 'rb').read()
text = data.decode('cp437')
# cursor-free transcript: drop SGR and other CSI sequences, handle \b
text = re.sub(r'\x1b\[[0-9;?]*[A-Za-z]', lambda m: '\n<CLS>\n' if m.group(0) == '\x1b[2J' else '', text)
out = []
for ch in text:
    if ch == '\b':
        if out and out[-1] != '\n':
            out.pop()
    elif ch == '\r':
        continue
    else:
        out.append(ch)
sys.stdout.write(''.join(out))
