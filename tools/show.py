#!/usr/bin/env python3
"""Print functions from the annotated listing: show.py ASMDIR SEG:OFF ..."""
import sys, re, os
d = sys.argv[1]
for a in sys.argv[2:]:
    seg = a.split(':')[0]
    txt = open(os.path.join(d, 'seg_%s.asm' % seg)).read()
    parts = txt.split('\n;; ===== ')
    for p in parts:
        if p.startswith(a):
            print(';; ===== ' + p.rstrip())
            break
    else:
        print(';; not found', a)
