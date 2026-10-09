#!/usr/bin/env python3
"""Collect field accesses on Empire / Pirate records from lifted code.
Heuristic: a variable assigned (g_c92e + (... * 0x2fc)) is an empire
pointer within that function; also direct *w((g_c92e + (...)) + OFF)."""
import re, sys, glob, collections
REC = {'empire': ('g_c92e', 0x2fc), 'pirate': ('g_c932', 0x106)}
which = sys.argv[2]
base, size = REC[which]
fields = collections.defaultdict(set)   # off -> set(size letters)
users = collections.defaultdict(set)
for path in glob.glob(sys.argv[1] + '/seg_*.c'):
    txt = open(path).read()
    for fn in re.split(r'\n(?=// [0-9a-f]{4}:)', txt):
        m = re.match(r'// ([0-9a-f:]+)', fn)
        name = m.group(1) if m else '?'
        ptrs = set()
        for mm in re.finditer(r'(\w+) = \(%s \+ \(\(?[^;]*\* %s\)\);' % (base, hex(size)), fn):
            ptrs.add(mm.group(1))
        for mm in re.finditer(r'\*([bwd])\(\(%s \+ \([^;]*?\* %s\)\) \+ (0x[0-9a-f]+|\d+)\)' % (base, hex(size)), fn):
            fields[int(mm.group(2), 0)].add(mm.group(1)); users[int(mm.group(2), 0)].add(name)
        for p in ptrs:
            for mm in re.finditer(r'\b%s->([bwdp])([0-9a-f]+)' % re.escape(p), fn):
                fields[int(mm.group(2), 16)].add(mm.group(1)); users[int(mm.group(2), 16)].add(name)
for off in sorted(fields):
    print('%04x %-4s %s' % (off, ''.join(sorted(fields[off])), ' '.join(sorted(users[off]))[:150]))
