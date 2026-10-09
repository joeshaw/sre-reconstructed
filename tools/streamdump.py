#!/usr/bin/env python3
"""List the objects in an SRE stream file (Data/Event.II, Report.II, ...):
class name, size and decrypted data.
usage: streamdump.py FILE [--hex]"""
import sys, os, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sredata import decrypt


def objects(d):
    pos = 0
    while pos + 0x50 <= len(d):
        size = struct.unpack_from('<h', d, pos)[0]
        nsalt = d[pos + 10:pos + 18]
        name, _ = decrypt(d[pos + 26:pos + 56], nsalt)
        dsalt = d[pos + 64:pos + 72]
        data, _ = decrypt(d[pos + 80:pos + 80 + size], dsalt)
        yield name.split(b'\0')[0].decode('latin-1'), data
        pos += 0x50 + size


def main():
    d = open(sys.argv[1], 'rb').read()
    for name, data in objects(d):
        print('%-20s %4d' % (name, len(data)))
        if '--hex' in sys.argv:
            print('   ', data.hex(' '))
        else:
            text = ''.join(chr(c) if 32 <= c < 127 else '.' for c in data)
            print('   ', text)


if __name__ == '__main__':
    main()
