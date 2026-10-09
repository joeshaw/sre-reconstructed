#!/usr/bin/env python3
"""Decrypt an SRE game directory's Data/*.II files.
usage: sredata.py DIR [--dump OUTDIR]  -- prints salts, writes plaintext."""
import sys, os, struct

def find(d, name):
    for f in os.listdir(d):
        if f.lower() == name.lower():
            return os.path.join(d, f)
    return None

def decrypt(data, salt):
    k = bytearray(salt); p = bytearray(data); f = 0
    for i in range(len(p)):
        j = i & 7
        p[i] ^= k[j]
        k[j] = ((k[j] >> 5) | (k[j] << 3)) & 0xff
        k[j] ^= f
        k[j] ^= p[i]
        f = (f + 7) & 0xff
    return bytes(p), bytes(k)

def cks(b):
    s, c, salt, key = struct.unpack('<ii8s8s', b)
    return salt, key

def main():
    d = os.path.join(sys.argv[1], 'DATA') if find(sys.argv[1], 'DATA') else sys.argv[1]
    d = find(sys.argv[1], 'DATA') or sys.argv[1]
    out = sys.argv[3] if len(sys.argv) > 3 and sys.argv[2] == '--dump' else None
    sysd = open(find(d, 'System.II'), 'rb').read()
    gsalt, gkey = cks(sysd[0x12a:0x142])
    gal, k = decrypt(open(find(d, 'Galaxy.II'), 'rb').read()[:0x3c6], gsalt)
    print('galaxy salt', gsalt.hex(), 'ok' if k == gkey else 'BADKEY')
    emp = open(find(d, 'Empire.II'), 'rb').read()
    pir = open(find(d, 'Pirate.II'), 'rb').read()
    eo = b''; po = b''
    for i in range(25):
        salt, key = cks(gal[0x54 + i * 0x18:0x54 + i * 0x18 + 0x18])
        p, k = decrypt(emp[i * 0x2fc:(i + 1) * 0x2fc], salt)
        eo += p
        print('empire', i, salt.hex(), 'ok' if k == key else 'BADKEY')
    for i in range(9):
        salt, key = cks(gal[0x2ac + i * 0x18:0x2ac + i * 0x18 + 0x18])
        p, k = decrypt(pir[i * 0x106:(i + 1) * 0x106], salt)
        po += p
        print('pirate', i, salt.hex(), 'ok' if k == key else 'BADKEY')
    if out:
        os.makedirs(out, exist_ok=True)
        open(os.path.join(out, 'system'), 'wb').write(sysd)
        open(os.path.join(out, 'galaxy'), 'wb').write(gal)
        open(os.path.join(out, 'empire'), 'wb').write(eo)
        open(os.path.join(out, 'pirate'), 'wb').write(po)

if __name__ == "__main__":
    main()
