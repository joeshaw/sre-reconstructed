#!/usr/bin/env python3
"""Compare two PLAYER.II files.  Bytes 0x43-0x61 of each record are left
as they were on the original's stack (only the first byte of that field
is ever set), so they are not compared."""
import sys
a = open(sys.argv[1], 'rb').read()
b = open(sys.argv[2], 'rb').read()
if len(a) != len(b):
    sys.exit(1)
for r in range(0, len(a), 0x72):
    x, y = bytearray(a[r:r + 0x72]), bytearray(b[r:r + 0x72])
    x[0x43:0x62] = y[0x43:0x62] = bytes(0x1f)
    if x != y:
        sys.exit(1)
