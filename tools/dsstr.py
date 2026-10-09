#!/usr/bin/env python3
"""Print C strings / bytes from the de-overlaid image's DGROUP.
usage: dsstr.py EXE OFF[:N] ...   (OFF hex; :N dumps N raw bytes)"""
import sys, struct
d = open(sys.argv[1], 'rb').read()
hdr = struct.unpack('<H', d[8:10])[0] * 16
base = hdr + (0x2d55 - 0x1000) * 16
for a in sys.argv[2:]:
    if ':' in a:
        o, n = a.split(':')
        o = int(o, 16)
        n = int(n, 0)
        b = d[base + o:base + o + n]
        print('%04x: %s' % (o, ' '.join('%02x' % x for x in b)))
    else:
        o = int(a, 16)
        b = d[base + o:base + o + 400]
        print('%04x: %r' % (o, b.split(b'\0')[0].decode('cp437')))
