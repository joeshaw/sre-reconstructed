#!/usr/bin/env python3
"""Scan a DGROUP range for vtables (runs of far code pointers), resolving
overlay stubs.  usage: vtables.py EXE START END"""
import sys, struct
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from disasm import load
mem = load(sys.argv[1])
DS = 0x2d55
start, end = int(sys.argv[2], 16), int(sys.argv[3], 16)
def resolve(s, o):
    a = s * 16 + o
    if a + 5 <= len(mem) and mem[a] == 0xea:
        o2, s2 = struct.unpack('<HH', mem[a + 1:a + 5])
        return s2, o2
    return s, o
import os, glob
SEGS = set(int(os.path.basename(f)[4:8], 16) for f in glob.glob(os.path.join(os.path.dirname(sys.argv[1]), 'asm', 'seg_*.asm')))
SEGS |= set(range(0x2c64, 0x2d55))
def is_code(s):
    return s in SEGS
o = start
while o < end:
    run = []
    p = o
    while True:
        off, seg = struct.unpack('<HH', mem[DS * 16 + p:DS * 16 + p + 4])
        if not is_code(seg) or off > 0x8000:
            break
        run.append(resolve(seg, off))
        p += 4
    if len(run) >= 3:
        print('%04x: %s' % (o, ' '.join('%04x:%04x' % x for x in run)))
        o = p
    else:
        o += 2
