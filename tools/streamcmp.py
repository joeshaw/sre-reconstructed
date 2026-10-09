#!/usr/bin/env python3
"""Compare two SRE stream files (Data/Event.II, Report.II, ...) object by
object, decrypted.  Bytes the original leaves as uninitialized memory are
not compared: the unused bits of a message's flags, and what follows the
name of a bank investment.
usage: streamcmp.py FILE1 FILE2  (exit status 1 if they differ)"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from streamdump import objects


def masked(name, data):
    d = bytearray(data)
    if name == 'Message' and len(d) > 0x82:
        d[0x82] &= 0x03
    if name == 'EventBank' and len(d) >= 0x64:
        end = d.index(0, 0x16, 0x64) if 0 in d[0x16:0x64] else 0x64
        d[end:0x64] = bytes(0x64 - end)
    return bytes(d)


def main():
    a = list(objects(open(sys.argv[1], 'rb').read()))
    b = list(objects(open(sys.argv[2], 'rb').read()))
    if len(a) != len(b):
        print('%d objects against %d' % (len(a), len(b)))
        sys.exit(1)
    for i, ((n1, d1), (n2, d2)) in enumerate(zip(a, b)):
        if n1 != n2 or masked(n1, d1) != masked(n2, d2):
            print('object %d (%s) differs' % (i, n1))
            sys.exit(1)


if __name__ == '__main__':
    main()
