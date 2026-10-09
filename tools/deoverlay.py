#!/usr/bin/env python3
"""Turn SRE.EXE (Borland C++ 3.1 MZ + VROOMM overlays) into a plain MZ EXE.

Each overlay module's code is appended to the load image as its own
segment; its segment fixups become ordinary MZ relocations, and the
INT 3Fh stub entries are rewritten as JMP FAR to the overlay code.
The result is meant for static analysis (Ghidra's MZ loader), not to run.
Also writes a .map file describing segments.
"""
import struct, sys

def main(exe, out):
    d = open(exe, 'rb').read()
    (sig, lastp, pages, nrel, hdrpara, minalloc, maxalloc, ss, sp, csum,
     ip, cs, reloff, ovno) = struct.unpack('<14H', d[:28])
    imgsize = (pages - 1) * 512 + (lastp or 512)
    hdr = hdrpara * 16
    img = bytearray(d[hdr:imgsize])
    relocs = [struct.unpack('<HH', d[reloff + i * 4:reloff + i * 4 + 4]) for i in range(nrel)]
    assert d[imgsize:imgsize + 4] == b'FBOV'
    ovsize, segtab, nseg = struct.unpack('<III', d[imgsize + 4:imgsize + 16])
    ovbase = imgsize + 16
    segs = [struct.unpack('<4H', d[segtab + i * 8:segtab + i * 8 + 8]) for i in range(nseg)]
    # bss/stack extend beyond the file image; reserve them so appended
    # overlay segments don't collide with uninitialised data.
    top = 0
    for seg, maxoff, flags, minoff in segs:
        end = seg + (((maxoff if maxoff != 0xffff else 0) + 16) >> 4)
        top = max(top, end)
    top = max(top, (len(img) + 15) >> 4)
    img.extend(bytes(top * 16 - len(img)))
    mapl = []
    for i, (seg, maxoff, flags, minoff) in enumerate(segs):
        kind = {0: 'data', 1: 'code', 3: 'stub', 4: 'other'}.get(flags, '?')
        line = '%2d seg=%04x size=%04x flags=%x %s' % (i, seg, maxoff, flags, kind)
        if flags == 3:
            so = seg * 16
            h = img[so:so + 0x20]
            fileoff, codesize, relsize, nent = struct.unpack('<IHHH', h[4:14])
            code = d[ovbase + fileoff:ovbase + fileoff + codesize]
            fix = d[ovbase + fileoff + codesize:ovbase + fileoff + codesize + relsize]
            newseg = (len(img) + 15) >> 4
            img.extend(bytes(newseg * 16 - len(img)))
            # fixups point at words holding a segment-table selector
            # (index * 8); replace with the segment's paragraph and add
            # an MZ relocation for it.
            code = bytearray(code)
            for j in range(0, relsize, 2):
                fo = struct.unpack('<H', fix[j:j + 2])[0]
                sel = struct.unpack('<H', code[fo:fo + 2])[0]
                assert sel % 8 == 0 and sel // 8 < nseg, (i, hex(fo), hex(sel))
                struct.pack_into('<H', code, fo, segs[sel // 8][0])
                relocs.append((fo, newseg))
            img.extend(code)
            img.extend(bytes((-len(img)) % 16))
            ents = []
            for e in range(nent):
                eo = so + 0x20 + e * 5
                assert img[eo:eo + 2] == b'\xcd\x3f'
                off = struct.unpack('<H', img[eo + 2:eo + 4])[0]
                img[eo:eo + 5] = b'\xea' + struct.pack('<HH', off, newseg)
                relocs.append((0x20 + e * 5 + 3, seg))
                ents.append('%04x' % off)
            line += ' -> ovl seg=%04x codesize=%04x entries=%s' % (newseg, codesize, ','.join(ents))
        mapl.append(line)
    nrel = len(relocs)
    hdrsize = 0x40 + nrel * 4
    hdrsize = (hdrsize + 511) & ~511
    total = hdrsize + len(img)
    h = bytearray(hdrsize)
    struct.pack_into('<14H', h, 0, 0x5a4d, total % 512, (total + 511) // 512, nrel,
                     hdrsize // 16, 0, 0xffff, ss, sp, 0, ip, cs, 0x40, 0)
    for i, (o, s) in enumerate(relocs):
        struct.pack_into('<HH', h, 0x40 + i * 4, o, s)
    open(out + '.exe', 'wb').write(bytes(h) + bytes(img))
    open(out + '.map', 'w').write('\n'.join(mapl) + '\n')

if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
