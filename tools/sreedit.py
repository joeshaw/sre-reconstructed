#!/usr/bin/env python3
"""Edit fields of an SRE game's galaxy and empire records (for building
test fixtures), keeping the files' checksums valid.

usage: sreedit.py DIR FIELD=VALUE...
  FIELD is galaxy.OFF or empire.N.OFF (OFF in hex); VALUE is TYPE:V with
  TYPE one of i8, i16, i32 or str (e.g. empire.0.194=i16:5)."""
import sys, os, struct, zlib
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sredata import find, decrypt

def encrypt(plain, salt):
    k = bytearray(salt); p = bytearray(plain); f = 0
    for i in range(len(p)):
        j = i & 7
        o = p[i]
        p[i] ^= k[j]
        k[j] = ((k[j] >> 5) | (k[j] << 3)) & 0xff
        k[j] ^= f
        k[j] ^= o
        f = (f + 7) & 0xff
    return bytes(p), bytes(k)

def checksum(plain, salt):
    s1 = s2 = 0
    for b in plain:
        s1 = (s1 + b) & 0xffff
        s2 = (s2 + s1) & 0xffff
    crc = zlib.crc32(plain) ^ 0xffffffff
    enc, key = encrypt(plain, salt)
    return enc, struct.pack('<II8s8s', s2, crc, salt, key)

def setfield(buf, off, val):
    t, v = val.split(':', 1)
    if t == 'str':
        b = v.encode('latin-1') + b'\0'
        buf[off:off + len(b)] = b
    else:
        fmt = {'i8': '<b', 'i16': '<h', 'i32': '<i'}[t]
        struct.pack_into(fmt, buf, off, int(v, 0))

def main():
    d = find(sys.argv[1], 'DATA') or sys.argv[1]
    sysp = find(d, 'System.II'); galp = find(d, 'Galaxy.II'); empp = find(d, 'Empire.II')
    sysd = bytearray(open(sysp, 'rb').read())
    gsalt = bytes(sysd[0x132:0x13a])
    graw = open(galp, 'rb').read()
    gal, _ = decrypt(graw[:0x3c6], gsalt)
    gal = bytearray(gal)
    emp = bytearray(open(empp, 'rb').read())
    plains = []
    for i in range(25):
        salt = bytes(gal[0x54 + i * 0x18 + 8:0x54 + i * 0x18 + 16])
        p, _ = decrypt(emp[i * 0x2fc:(i + 1) * 0x2fc], salt)
        plains.append(bytearray(p))
    for a in sys.argv[2:]:
        f, val = a.split('=', 1)
        parts = f.split('.')
        if parts[0] == 'galaxy':
            setfield(gal, int(parts[1], 16), val)
        else:
            setfield(plains[int(parts[1])], int(parts[2], 16), val)
    for i in range(25):
        salt = bytes(gal[0x54 + i * 0x18 + 8:0x54 + i * 0x18 + 16])
        enc, ck = checksum(bytes(plains[i]), salt)
        emp[i * 0x2fc:(i + 1) * 0x2fc] = enc
        gal[0x54 + i * 0x18:0x54 + i * 0x18 + 0x18] = ck
    genc, gck = checksum(bytes(gal), gsalt)
    sysd[0x12a:0x142] = gck
    open(galp, 'wb').write(genc + graw[0x3c6:])
    open(empp, 'wb').write(bytes(emp))
    open(sysp, 'wb').write(bytes(sysd))

main()
