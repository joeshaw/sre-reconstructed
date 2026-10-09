#!/usr/bin/env python3
"""Annotated disassembler for the de-overlaid SRE image (see deoverlay.py).

Produces one listing per code segment with:
  * functions split at known entry points (Ghidra's list + call targets),
  * far calls through overlay stubs resolved to the real target,
  * names from names.txt ("SEG:OFF name") and globals.txt ("OFF name"),
  * far-pointer pushes of DGROUP strings shown as string literals,
  * [bp+N] shown as argN / [bp-N] as locN.

Addresses use the same segment numbering as Ghidra (image loaded at 1000h).
"""
import struct, sys, os, re, argparse
import capstone

LOAD = 0x1000
DGROUP = 0x2d55

def load(path):
    d = open(path, 'rb').read()
    hdrpara = struct.unpack('<H', d[8:10])[0]
    nrel = struct.unpack('<H', d[6:8])[0]
    reloff = struct.unpack('<H', d[24:26])[0]
    img = bytearray(d[hdrpara * 16:])
    for i in range(nrel):
        o, s = struct.unpack('<HH', d[reloff + i * 4:reloff + i * 4 + 4])
        a = s * 16 + o
        v = struct.unpack('<H', img[a:a + 2])[0]
        img[a:a + 2] = struct.pack('<H', (v + LOAD) & 0xffff)
    mem = bytearray(LOAD * 16) + img
    return mem

def read_names(path):
    names = {}
    if os.path.exists(path):
        for line in open(path):
            line = line.split('#')[0].strip()
            if not line:
                continue
            parts = line.split(None, 1)
            seg, off = parts[0].split(':')
            names[(int(seg, 16), int(off, 16))] = parts[1].strip() if len(parts) > 1 else ''
    return names

def read_globals(path):
    g = {}
    if os.path.exists(path):
        for line in open(path):
            line = line.split('#')[0].strip()
            if not line:
                continue
            off, name = line.split(None, 1)
            g[int(off, 16)] = name.strip()
    return g

class Dis:
    def __init__(self, mem, names, globs, segments):
        self.mem = mem
        self.names = names
        self.globs = globs
        self.segments = segments       # sorted list of (seg, size)
        self.md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_16)
        self.md.detail = False
        self.switches = {}

    def lin(self, seg, off):
        return seg * 16 + off

    def u16(self, seg, off):
        a = self.lin(seg, off)
        return struct.unpack('<H', self.mem[a:a + 2])[0]

    def cstr(self, off, maxlen=200):
        a = DGROUP * 16 + off
        if a >= len(self.mem):
            return None
        b = self.mem[a:a + maxlen]
        z = b.find(b'\0')
        if z < 0:
            return None
        s = bytes(b[:z])
        if any((c < 32 and c not in (9, 10, 13, 27)) for c in s):
            return None
        return s.decode('cp437')

    def resolve_far(self, seg, off):
        """Follow overlay stub JMP FAR."""
        a = self.lin(seg, off)
        if self.mem[a] == 0xea:
            o2, s2 = struct.unpack('<HH', self.mem[a + 1:a + 5])
            return (s2, o2)
        return (seg, off)

    def name_of(self, seg, off):
        n = self.names.get((seg, off))
        if n:
            return n.split('(')[0].strip()
        return 'f_%04x_%04x' % (seg, off)

    def insns(self, seg, start, end):
        code = bytes(self.mem[self.lin(seg, start):self.lin(seg, end)])
        return list(self.md.disasm(code, start))

    def func_body(self, seg, entry, limit):
        """Recursive descent within [entry, limit)."""
        seen = {}
        todo = [entry]
        targets = set()
        while todo:
            pc = todo.pop()
            while pc < limit and pc not in seen:
                code = bytes(self.mem[self.lin(seg, pc):self.lin(seg, pc) + 16])
                ins = next(self.md.disasm(code, pc), None)
                if ins is None:
                    break
                seen[pc] = ins
                m = ins.mnemonic
                nxt = pc + ins.size
                if m.startswith('j') or m in ('loop', 'loope', 'loopne', 'jcxz'):
                    op = ins.op_str
                    if m == 'jmp' and ('ptr' in op or ':' in op or op.startswith(('ax', 'bx', 'cx', 'dx', 'si', 'di'))):
                        # indirect / far jump: switch table?
                        tm = re.match(r'word ptr cs:\[(bx|di|si) \+ 0x([0-9a-f]+)\]', op)
                        if tm:
                            tab = int(tm.group(2), 16)
                            srch = self._search_switch(seg, pc, seen)
                            if srch:
                                stab, n = srch
                                vals = [self.u16(seg, stab + i * 2) for i in range(n)]
                                tgts = [self.u16(seg, stab + 2 * n + i * 2) for i in range(n)]
                                for t in tgts:
                                    targets.add(t)
                                    todo.append(t)
                                self.switches[(seg, pc)] = ('search', vals, tgts)
                            else:
                                n = self._switch_count(seg, pc, seen)
                                for i in range(n):
                                    t = self.u16(seg, tab + i * 2)
                                    targets.add(t)
                                    todo.append(t)
                                self.switches[(seg, pc)] = (tab, n)
                        break
                    try:
                        t = int(op, 16)
                    except ValueError:
                        break
                    targets.add(t)
                    todo.append(t)
                    if m == 'jmp':
                        break
                elif m in ('ret', 'retf', 'iret'):
                    break
                elif m == 'int' and ins.op_str == '0x20':
                    break
                pc = nxt
        return seen, targets

    def _search_switch(self, seg, pc, seen):
        # Borland linear search switch:
        #   mov cx, N / mov bx, T / L: mov ax, cs:[bx] / cmp ... / je F /
        #   add bx, 2 / loop L / jmp default / F: jmp cs:[bx + 2N]
        prev = sorted(a for a in seen if a < pc)[-12:]
        n = t = None
        for a in prev:
            ins = seen[a]
            m = re.match(r'^cx, (0x[0-9a-f]+|\d+)$', ins.op_str)
            if ins.mnemonic == 'mov' and m:
                n = int(m.group(1), 0)
            m = re.match(r'^bx, (0x[0-9a-f]+|\d+)$', ins.op_str)
            if ins.mnemonic == 'mov' and m:
                t = int(m.group(1), 0)
            if ins.mnemonic == 'loop':
                if n and t is not None:
                    return (t, n)
        return None

    def _switch_count(self, seg, pc, seen):
        # look back for "cmp reg, imm" followed by ja
        prev = sorted(a for a in seen if a < pc)[-6:]
        for a in reversed(prev):
            ins = seen[a]
            if ins.mnemonic == 'cmp':
                m = re.match(r'\w+, (0x[0-9a-f]+|\d+)$', ins.op_str)
                if m:
                    return int(m.group(1), 0) + 1
        return 1

    def fmt_operand(self, op, far):
        def bpsub(m):
            sign, val = m.group(1), int(m.group(2), 0)
            if sign == '+':
                base = 6 if far else 4
                if val >= base:
                    return '[arg_%x]' % (val - base)
                return '[bp+%x]' % val
            return '[loc_%x]' % val
        op = re.sub(r'\[bp ([+-]) (0x[0-9a-f]+|\d+)\]', bpsub, op)
        def gsub(m):
            off = int(m.group(1), 16)
            n = self.globs.get(off)
            if n:
                return '[%s]' % n
            return '[g_%04x]' % off
        op = re.sub(r'(?<![:\w])\[(0x[0-9a-f]+)\]', gsub, op)
        return op

    def listing(self, seg, entry, limit, far_hint=None):
        body, targets = self.func_body(seg, entry, limit)
        if not body:
            return [], entry
        addrs = sorted(body)
        far = any(body[a].mnemonic == 'retf' for a in addrs)
        out = []
        pushes = []   # recent push values for string detection
        movax = None
        end = max(a + body[a].size for a in addrs)
        for a in addrs:
            ins = body[a]
            if a in targets and a != entry:
                out.append('L_%04x:' % a)
            m, op = ins.mnemonic, ins.op_str
            note = ''
            if m == 'lcall' and re.match(r'^0x[0-9a-f]+, 0x[0-9a-f]+$', op):
                m = 'call'
                op = op.replace(', ', ':')
            if m == 'ljmp' and re.match(r'^0x[0-9a-f]+, 0x[0-9a-f]+$', op):
                m = 'jmp'
                op = op.replace(', ', ':')
            if m == 'call':
                if op.startswith('0x') and ':' in op:
                    s, o = [int(x, 16) for x in op.split(':')]
                    s2, o2 = self.resolve_far(s, o)
                    op = self.name_of(s2, o2)
                    if (s2, o2) != (s, o):
                        note = 'via stub %04x:%04x' % (s, o)
                elif op.startswith('0x'):
                    t = int(op, 16)
                    op = self.name_of(seg, t) + ' (near)'
                else:
                    op = self.fmt_operand(op, far)
            elif m.startswith('j') or m.startswith('loop'):
                if op.startswith('0x') and ':' not in op:
                    op = 'L_%04x' % int(op, 16)
                elif ':' in op and op.startswith('0x'):
                    s, o = [int(x, 16) for x in op.split(':')]
                    op = self.name_of(s, o)
                else:
                    op = self.fmt_operand(op, far)
            else:
                op = self.fmt_operand(op, far)
            # string detection: push imm after push ds / push DGROUP
            if m == 'push':
                v = None
                if re.match(r'^(0x[0-9a-f]+|\d+)$', ins.op_str):
                    v = int(ins.op_str, 0)
                elif ins.op_str == 'ax' and movax is not None:
                    v = movax
                if ins.op_str == 'ds':
                    pushes.append(('seg', DGROUP))
                elif v is not None:
                    if pushes and pushes[-1] == ('seg', DGROUP) and v != DGROUP:
                        s = self.cstr(v)
                        if v in self.globs:
                            note = '&' + self.globs[v]
                        elif s:
                            note = repr(s)
                        else:
                            note = '&g_%04x' % v
                    pushes.append(('seg', DGROUP) if v == DGROUP else ('val', v))
                else:
                    pushes.append(('reg', ins.op_str))
            if m == 'mov' and re.match(r'^ax, (0x[0-9a-f]+|\d+)$', ins.op_str):
                movax = int(ins.op_str.split(', ')[1], 0)
            elif re.match(r'^(ax|al|ah)\b', ins.op_str) and m not in ('push', 'cmp', 'test'):
                movax = None
            if False:
                v = int(ins.op_str.split(', ')[1], 0)
                s = self.cstr(v) if v > 0x80 else None
                if s is not None and len(s) >= 2:
                    note = '?' + repr(s)
            if (seg, a) in self.switches:
                sw = self.switches[(seg, a)]
                if sw[0] == 'search':
                    note = 'switch: ' + ' '.join('%d:L_%04x' % (v if v < 0x8000 else v - 0x10000, t)
                                                 for v, t in zip(sw[1], sw[2]))
                else:
                    tab, n = sw
                    note = 'switch table @%04x: ' % tab + ' '.join(
                        '%d:L_%04x' % (i, self.u16(seg, tab + i * 2)) for i in range(n))
            line = '  %04x  %-8s %s' % (a, m, op)
            if note:
                line = '%-60s ; %s' % (line, note)
            out.append(line)
            if m in ('ret', 'retf', 'jmp'):
                pushes = []
        return out, end


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe')
    ap.add_argument('funcs')
    ap.add_argument('outdir')
    ap.add_argument('--names', default=os.path.join(os.path.dirname(__file__), 're', 'names.txt'))
    ap.add_argument('--globals', default=os.path.join(os.path.dirname(__file__), 're', 'globals.txt'))
    a = ap.parse_args()
    mem = load(a.exe)
    names = read_names(a.names)
    globs = read_globals(a.globals)
    entries = {}
    for line in open(a.funcs):
        addr = line.split()[0]
        s, o = [int(x, 16) for x in addr.split(':')]
        entries.setdefault(s, set()).add(o)
    for (s, o) in names:
        entries.setdefault(s, set()).add(o)
    dis = Dis(mem, names, globs, None)
    os.makedirs(a.outdir, exist_ok=True)
    for seg in sorted(entries):
        if seg == DGROUP:
            continue
        offs = sorted(entries[seg])
        # drop stub "functions" (JMP FAR thunks)
        lines = []
        for i, o in enumerate(offs):
            if mem[seg * 16 + o] == 0xea:
                continue
            limit = offs[i + 1] if i + 1 < len(offs) else 0x10000
            nm = names.get((seg, o))
            hdr = '\n;; ===== %04x:%04x %s' % (seg, o, nm if nm else 'f_%04x_%04x' % (seg, o))
            body, end = dis.listing(seg, o, limit)
            if not body:
                continue
            lines.append(hdr)
            lines.extend(body)
        if lines:
            open(os.path.join(a.outdir, 'seg_%04x.asm' % seg), 'w').write('\n'.join(lines) + '\n')

if __name__ == '__main__':
    main()
