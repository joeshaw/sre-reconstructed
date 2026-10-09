#!/usr/bin/env python3
"""Lift the Borland C++ code of sreflat.exe into readable pseudo-C.

This is a reading aid only.  Each basic block is executed symbolically:
register contents become expressions, pushes become call arguments,
memory writes and calls become statements, and conditional jumps become
'if (cond) goto L'.  Far pointers are rendered as p->wNN (word at NN),
p->bNN (byte), p->dNN (dword).
"""
import re, sys, os, argparse
sys.path.insert(0, os.path.dirname(__file__))
from disasm import load, read_names, read_globals, Dis, DGROUP

# register-argument / callee-pops helpers: name -> (kind, stack bytes popped by callee)
REGHELP = {
    'LXMUL': 'mul', 'N_LXMUL': 'mul',
    'LXLSH': 'shl', 'N_LXLSH': 'shl', 'LXRSH': 'sar', 'LXURSH': 'shr',
    'LDIV': 'div', 'N_LDIV': 'div', 'LMOD': 'mod', 'N_LMOD': 'mod',
    'SCOPY': 'scopy', 'PCMP': 'pcmp',
}

R16 = ['ax', 'bx', 'cx', 'dx', 'si', 'di', 'es', 'ds', 'ss', 'cs', 'bp', 'sp']
R8 = {'al': ('ax', 0), 'ah': ('ax', 1), 'bl': ('bx', 0), 'bh': ('bx', 1),
      'cl': ('cx', 0), 'ch': ('cx', 1), 'dl': ('dx', 0), 'dh': ('dx', 1)}

def num(v, size=16):
    if isinstance(v, str):
        return v
    if size == 16 and v >= 0xff00:
        return str(v - 0x10000)
    if size == 8 and v >= 0x80:
        return str(v - 0x100)
    if v < 10:
        return str(v)
    return hex(v)

def isnum(s):
    return re.match(r'^-?(0x[0-9a-f]+|\d+)$', s) is not None

def paren(e):
    if re.match(r'^[\w.>\-\[\]&*:"\']+$', e) and not e.startswith('-'):
        return e
    if e.startswith('(') and e.endswith(')') and balanced(e[1:-1]):
        return e
    return '(' + e + ')'

def balanced(s):
    d = 0
    for c in s:
        if c == '(':
            d += 1
        elif c == ')':
            d -= 1
            if d < 0:
                return False
    return d == 0

JCC = {'je': '==', 'jne': '!=', 'jl': '<', 'jle': '<=', 'jg': '>', 'jge': '>=',
       'jb': '<u', 'jbe': '<=u', 'ja': '>u', 'jae': '>=u'}

class Lifter:
    def __init__(self, dis, globs):
        self.dis = dis
        self.globs = globs
        self.tcount = 0
        self.sigcache = {}

    def gname(self, off):
        n = self.globs.get(off)
        return n if n else 'g_%04x' % off

    # ------------------------------------------------------------------
    def l32(self, hi, lo):
        h, l = self.rd(hi), self.rd(lo)
        if h == l + '.hi':
            return l
        if h.startswith('SX(') and h[3:-1] == l:
            return '(long)' + paren(l)
        if h == '0':
            return '(ulong)' + paren(l)
        return '%s:%s' % (h, l)

    def release(self, k):
        self.pending -= k
        if self.pending < 0:
            x = -self.pending
            if x:
                del self.stack[-x:]
            self.pending = 0

    def reset(self):
        self.pending = 0
        self.r = {k: k for k in R16}
        self.r8 = {}
        self.stack = []
        self.flags = None
        self.dirty = set()

    def rd(self, reg):
        if reg in R8:
            if reg in self.r8:
                return self.r8[reg]
            base, hi = R8[reg]
            v = self.r[base]
            if hi:
                return 'HI(%s)' % v
            return v if v != base else reg
        return self.r.get(reg, reg)

    def wr(self, reg, val):
        if reg in R8:
            base, hi = R8[reg]
            self.r8[reg] = val
            if not hi:
                self.r[base] = val
            elif val == '0':
                lo = {'ax': 'al', 'bx': 'bl', 'cx': 'cl', 'dx': 'dl'}[base]
                self.r[base] = self.r8.get(lo, self.r[base])
            else:
                self.r[base] = 'SETHI(%s,%s)' % (self.r[base], val)
            self.dirty.add(base)
            return
        self.r[reg] = val
        for k in list(self.r8):
            if R8[k][0] == reg:
                del self.r8[k]
        self.dirty.add(reg)

    def mem(self, op, lea=False):
        m = re.match(r'^(?:(byte|word|dword|qword|tbyte) )?ptr (?:(\w\w):)?\[(.*)\]$', op)
        if not m:
            m = re.match(r'^(?:(\w\w):)?\[(.*)\]$', op)
            if not m:
                return None
            size, seg, inner = None, m.group(1), m.group(2)
        else:
            size, seg, inner = m.group(1), m.group(2), m.group(3)
        terms = re.findall(r'([+-]?)\s*(\w+)', inner.replace(' ', ''))
        regs = []
        disp = 0
        for sgn, t in terms:
            if t in ('bx', 'si', 'di', 'bp'):
                regs.append(t)
            else:
                v = int(t, 0)
                disp += -v if sgn == '-' else v
        disp &= 0xffff
        pfx = {'byte': 'b', 'word': 'w', 'dword': 'd', None: 'p'}.get(size, 'q')
        if 'bp' in regs:
            regs.remove('bp')
            sd = disp - 0x10000 if disp >= 0x8000 else disp
            if sd >= self.argbase:
                nm = 'arg_%x' % (sd - self.argbase)
            elif sd >= 0:
                nm = 'bp_%x' % sd
            else:
                nm = 'loc_%x' % (-sd)
            if regs:
                idx = ' + '.join(self.rd(x) for x in regs)
                e = '%s[%s]' % (nm, idx)
                return ('&' + e) if lea else e
            if lea:
                return '&' + nm
            return nm if pfx in ('w', 'p') else '%s.%s' % (nm, pfx)
        if not regs:
            if seg in (None, 'ds'):
                nm = self.gname(disp)
                if lea:
                    return '&' + nm
                return nm if pfx in ('w', 'p') else '%s.%s' % (nm, pfx)
            return '%s:[%s]' % (seg, num(disp))
        base = ' + '.join(self.rd(x) for x in regs)
        if lea:
            if disp:
                return '(%s + %s)' % (base, num(disp))
            return base
        if seg in ('ss',) or base.startswith('&') or base.startswith('('):
            if disp:
                return '*%s(%s + %s)' % (pfx, base, num(disp))
            return '*%s%s' % (pfx, paren(base))
        return '%s->%s%x' % (paren(base), pfx, disp)

    def val(self, op, size=16):
        op = op.strip()
        if op in self.r or op in R8:
            return self.rd(op)
        if isnum(op):
            return num(int(op, 0), size)
        m = self.mem(op)
        if m is not None:
            return m
        return op

    def opsize(self, op, other=None):
        if op in R8 or (other and other in R8) or op.startswith('byte'):
            return 8
        return 16

    # ------------------------------------------------------------------
    def emit(self, s):
        self.out.append('    ' + s)

    def materialize(self, name):
        # registers holding expressions that reference memory 'name' get
        # pinned before that memory changes
        base = name[:-2] if re.search(r'\.[bwd]$', name) else name
        pat = r'(?<![\w>])%s(?![\w])' % re.escape(base)
        for k in ('ax', 'bx', 'cx', 'dx', 'si', 'di', 'es'):
            v = self.r[k]
            if v != k and re.search(pat, v):
                self.emit('%s = %s;' % (k, v))
                self.r[k] = k
        for k in list(self.r8):
            if re.search(pat, self.r8[k]):
                del self.r8[k]

    def store(self, dst, expr, op='='):
        if dst in self.r or dst in R8:
            if op == '=':
                self.wr(dst, expr)
            else:
                cur = self.rd(dst)
                self.wr(dst, '(%s %s %s)' % (cur, op[:-1], paren(expr)))
            return
        d = self.val(dst)
        self.materialize(d)
        self.emit('%s %s %s;' % (d, op, expr))

    def setflags(self, kind, a, b=None):
        self.flags = (kind, a, b)

    def cond(self, m):
        if not self.flags:
            return '%s?' % m
        kind, a, b = self.flags
        if m in ('js', 'jns'):
            e = a if kind == 'res' else ('%s - %s' % (a, b) if kind == 'cmp' else '%s & %s' % (a, b))
            return '%s %s 0' % (paren(e), '<' if m == 'js' else '>=')
        op = JCC.get(m)
        if op is None:
            return '%s(%s)' % (m, a)
        if kind == 'cmp':
            return '%s %s %s' % (a, op, b)
        if kind == 'test':
            e = a if a == b else '%s & %s' % (paren(a), paren(b))
            return '%s %s 0' % (paren(e), op)
        return '%s %s 0' % (paren(a), op)

    def combine_args(self, items):
        # items in C order (first arg first), each a word
        out = []
        i = 0
        while i < len(items):
            a = items[i]
            b = items[i + 1] if i + 1 < len(items) else None
            if b is not None:
                if b == 'ds' and isnum(a):
                    v = int(a, 0) & 0xffff
                    s = self.dis.cstr(v)
                    if v in self.globs:
                        out.append('&' + self.globs[v])
                    elif s is not None and len(s) > 0:
                        out.append(repr(s))
                    else:
                        out.append('&g_%04x' % v)
                    i += 2
                    continue
                if b in ('ss', 'ds') and a.startswith('&'):
                    out.append(a)
                    i += 2
                    continue
                if b == 'SEG(%s)' % a:
                    out.append(a)
                    i += 2
                    continue
                m1 = re.match(r'^(.*?)(\w)_?([0-9a-f]+)$', a)
                m2 = re.match(r'^(.*?)(\w)_?([0-9a-f]+)$', b)
                if m1 and m2 and m1.group(1) == m2.group(1) and m1.group(2) == m2.group(2):
                    va, vb = int(m1.group(3), 16), int(m2.group(3), 16)
                    pre = m1.group(1) + m1.group(2)
                    if pre.endswith('loc') or pre.endswith('loc_'):
                        if va == vb + 2:
                            out.append('L:' + a)
                            i += 2
                            continue
                    elif vb == va + 2:
                        out.append('L:' + (a.replace('->w', '->d') if '->w' in a else a))
                        i += 2
                        continue
                if b == 'SX(%s)' % a:
                    out.append('(long)' + paren(a))
                    i += 2
                    continue
                if b == a + '.hi':
                    out.append(a)
                    i += 2
                    continue
                if a == '0' and b == '0':
                    out.append('0L')
                    i += 2
                    continue
            out.append(a)
            i += 1
        return out

    def argwords(self, seg, off):
        key = (seg, off)
        if key in self.sigcache:
            return self.sigcache[key]
        n = None
        try:
            body, _ = self.dis.func_body(seg, off, 0x10000)
            far = any(i.mnemonic == 'retf' for i in body.values())
            base = 6 if far else 4
            mx = 0
            for i in body.values():
                for mm in re.finditer(r'(?:(byte|word|dword)? ?ptr )?\[bp \+ (0x[0-9a-f]+|\d+)\]', i.op_str):
                    v = int(mm.group(2), 0)
                    if v >= base:
                        sz = {'byte': 1, 'word': 2, 'dword': 4}.get(mm.group(1), 4 if i.mnemonic in ('les', 'lds') else 2)
                        mx = max(mx, v - base + sz)
                if i.mnemonic in ('ret', 'retf') and i.op_str:
                    pass
            n = (mx + 1) // 2
        except Exception:
            n = None
        self.sigcache[key] = n
        return n

    def do_call(self, target, ins_list, idx, near=False, addr=None):
        name = target
        # cleanup lookahead
        nwords = None
        j = idx + 1
        k = 0
        while j < len(ins_list):
            n = ins_list[j]
            if n.mnemonic == 'pop' and n.op_str == 'cx':
                k += 1
                j += 1
                continue
            if n.mnemonic == 'add' and n.op_str.startswith('sp, '):
                k += int(n.op_str.split(', ')[1], 0) // 2
                j += 1
            break
        helper = REGHELP.get(name)
        if helper in ('mul',):
            e = '(%s * %s)' % (self.l32('dx', 'ax'), self.l32('cx', 'bx'))
            self.wr('ax', e)
            self.wr('dx', e + '.hi')
            return 0
        if helper in ('shl', 'sar', 'shr'):
            op = {'shl': '<<', 'sar': '>>', 'shr': '>>>'}[helper]
            e = '(%s %s %s)' % (self.l32('dx', 'ax'), op, self.rd('cl'))
            self.wr('ax', e)
            self.wr('dx', e + '.hi')
            return 0
        if helper in ('div', 'mod'):
            args = list(reversed(self.stack[-4:]))
            del self.stack[-4:]
            a = self.combine_args(args)
            op = '/' if helper == 'div' else '%'
            e = '(%s %s %s)' % (paren(a[0]), op, paren(a[1] if len(a) > 1 else '?'))
            self.wr('ax', e)
            self.wr('dx', e + '.hi')
            return 0
        if helper == 'scopy':
            args = list(reversed(self.stack[-4:]))
            del self.stack[-4:]
            a = self.combine_args(args)
            self.emit('memcpy(%s, %s, %s);' % (a[1] if len(a) > 1 else '?', a[0], self.rd('cx')))
            return 0
        if helper == 'pcmp':
            self.setflags('cmp', '%s:%s' % (self.rd('dx'), self.rd('ax')), '%s:%s' % (self.rd('cx'), self.rd('bx')))
            return 0
        nc = self.argwords(*addr) if addr else None
        if nc is not None:
            n = nc
        else:
            n = k
        n = min(n, len(self.stack)) if n <= len(self.stack) else n
        args = list(reversed(self.stack[-n:])) if n else []
        if n > len(self.stack):
            args += ['?'] * (n - len(self.stack))
        if n:
            del self.stack[-n:]
        self.pending += n
        self.release(k)
        a = self.combine_args(args)
        regs = ''
        self.tcount += 1
        t = 't%d' % self.tcount
        self.emit('%s = %s(%s);%s' % (t, name, ', '.join(a), regs))
        self.r = {kk: kk for kk in R16}
        self.r8 = {}
        self.r['ax'] = t
        self.r['dx'] = t + '.hi'
        self.dirty = {'ax', 'dx'}
        self.flags = None
        return j - idx - 1

    def flush(self, target=None):
        if target is not None and target not in self.retblocks:
            return
        for k in ('dx', 'ax'):
            if k in self.dirty and self.r[k] != k:
                self.emit('%s = %s;' % (k, self.r[k]))

    def lift_func(self, seg, entry, limit):
        body, targets = self.dis.func_body(seg, entry, limit)
        if not body:
            return None
        addrs = sorted(body)
        far = any(body[a].mnemonic == 'retf' for a in addrs)
        self.argbase = 6 if far else 4
        # does the frame use push bp/mov bp,sp?
        ins_list = [body[a] for a in addrs]
        leaders = {entry} | set(targets)
        for i, ins in enumerate(ins_list):
            if ins.mnemonic.startswith('j') or ins.mnemonic.startswith('loop') or ins.mnemonic in ('ret', 'retf'):
                if i + 1 < len(ins_list):
                    leaders.add(ins_list[i + 1].address)
        # frame size
        frame = 0
        for ins in ins_list[:4]:
            if ins.mnemonic == 'sub' and ins.op_str.startswith('sp, '):
                frame = int(ins.op_str.split(', ')[1], 0)
        self.out = []
        self.reset()
        self.keepregs = False
        # blocks that reach a return without touching ax
        self.retblocks = set()
        RREG = re.compile(r'\b(ax|al|ah|dx|dl|dh)\b')
        def live_in(t, depth=0):
            if t not in body or depth > 4:
                return False
            j = addrs.index(t)
            for ins in ins_list[j:j + 12]:
                m, op = ins.mnemonic, ins.op_str
                ops = [o.strip() for o in op.split(', ')] if op else []
                if m in ('ret', 'retf'):
                    return True
                if m == 'jmp':
                    if isnum(op):
                        return live_in(int(op, 16), depth + 1)
                    return True
                if m in ('call', 'lcall'):
                    return False
                if m in ('mov', 'lea', 'les', 'lds') and len(ops) == 2:
                    if RREG.search(ops[1]):
                        return True
                    if ops[0] in ('ax', 'dx'):
                        return False
                    continue
                if m == 'xor' and len(ops) == 2 and ops[0] == ops[1] and ops[0] in ('ax', 'dx'):
                    return False
                if m in ('cwd', 'cdq', 'cbw', 'cwde', 'mul', 'imul', 'div', 'idiv'):
                    return True
                if RREG.search(op):
                    return True
            return False
        for t in leaders:
            if live_in(t):
                self.retblocks.add(t)
        i = 0
        prologue = True
        while i < len(ins_list):
            ins = ins_list[i]
            a = ins.address
            if a in leaders and a != entry and a not in targets:
                pass
            elif a in leaders and a != entry:
                prev = ins_list[i - 1].mnemonic if i else ''
                if prev not in ('jmp', 'ret', 'retf'):
                    self.flush(a)
                fl = self.flags if (prev in JCC or prev in ('js', 'jns')) else None
                self.reset()
                self.flags = fl
                self.out.append('L_%04x:' % a)
            m, op = ins.mnemonic, ins.op_str
            ops = [o.strip() for o in op.split(', ')] if op else []
            skip = self.step(seg, ins, m, ops, ins_list, i, prologue)
            if m not in ('push', 'mov', 'sub') or ops[:1] not in (['bp'], ['si'], ['di'], ['sp']):
                prologue = False
            i += 1 + (skip or 0)
        nm = self.dis.name_of(seg, entry)
        hdr = '// %04x:%04x %s  (%s, frame %d)' % (seg, entry, nm, 'far' if far else 'near', frame)
        return [hdr, '{'] + self.out + ['}']

    def step(self, seg, ins, m, ops, ins_list, i, prologue):
        a = ins.address
        if m == 'nop' or m == 'cld' or m == 'cli' or m == 'sti':
            return
        if prologue and m == 'push' and ops[0] in ('bp', 'si', 'di', 'ds'):
            return
        if m == 'mov' and ops == ['bp', 'sp']:
            return
        if m == 'sub' and ops[0] == 'sp' and prologue:
            return
        if m == 'push':
            o = ops[0]
            if o == 'cs' and i + 1 < len(ins_list) and ins_list[i + 1].mnemonic == 'call':
                return
            if o in ('ds', 'ss'):
                self.stack.append(o)
            elif o == 'es':
                self.stack.append(self.rd('es'))
            else:
                self.stack.append(self.val(o))
            return
        if m == 'pop':
            o = ops[0]
            if self.stack:
                v = self.stack.pop()
                if o in ('si', 'di', 'bp', 'ds') and v == o:
                    return
                self.wr(o, v)
            return
        if m == 'sub' and ops[0] == 'sp':
            n = int(ops[1], 0) // 2
            self.stack.extend(['TMP'] * n)
            return
        if m == 'add' and ops[0] == 'sp':
            self.release(int(ops[1], 0) // 2)
            return
        if m in ('call', 'lcall'):
            o = ins.op_str
            if re.match(r'^0x[0-9a-f]+, 0x[0-9a-f]+$', o):
                s, off = [int(x, 16) for x in o.split(', ')]
                s2, o2 = self.dis.resolve_far(s, off)
                return self.do_call(self.dis.name_of(s2, o2), ins_list, i, addr=(s2, o2))
            if re.match(r'^(0x[0-9a-f]+|\d+)$', o):
                return self.do_call(self.dis.name_of(seg, int(o, 0)), ins_list, i, near=True, addr=(seg, int(o, 0)))
            mm = self.mem(o)
            tgt = '(*%s)' % mm if mm else o
            if mm and '->' in mm:
                b, f = mm.rsplit('->', 1)
                tgt = 'VCALL(%s, %s)' % (b, '0x' + f[1:])
            return self.do_call(tgt, ins_list, i)
        if m in ('ret', 'retf'):
            self.flush_ret()
            return
        if m == 'leave':
            return
        if m == 'mov':
            d, s = ops
            if d in ('sp',):
                return
            if s == 'sp':
                return
            sz = self.opsize(d, s)
            self.store(d, self.val(s, sz))
            return
        if m in ('les', 'lds'):
            d, s = ops
            v = self.mem(s)
            if v and v.endswith('.p'):
                v = v[:-2]
            self.wr(d, v)
            self.wr('es' if m == 'les' else 'ds', 'SEG(%s)' % v)
            return
        if m == 'lea':
            d, s = ops
            self.wr(d, self.mem(s, lea=True))
            return
        if m == 'xchg':
            x, y = ops
            vx, vy = self.val(x), self.val(y)
            if x in self.r and y in self.r:
                self.wr(x, vy)
                self.wr(y, vx)
            else:
                self.store(x, vy)
                self.store(y, vx)
            return
        if m in ('add', 'sub', 'and', 'or', 'xor', 'adc', 'sbb'):
            d, s = ops
            sz = self.opsize(d, s)
            if m == 'xor' and d == s:
                self.store(d, '0')
                self.setflags('res', '0')
                return
            if m == 'or' and d == s:
                self.setflags('res', self.val(d))
                return
            if m == 'and' and d == s:
                self.setflags('res', self.val(d))
                return
            opc = {'add': '+', 'sub': '-', 'and': '&', 'or': '|', 'xor': '^', 'adc': '+c', 'sbb': '-c'}[m]
            sv = self.val(s, sz)
            if m in ('adc', 'sbb'):
                sv = '%s + CARRY' % sv if m == 'adc' else '%s + BORROW' % sv
                opc = opc[0]
            if d in self.r or d in R8:
                cur = self.rd(d)
                res = '%s %s %s' % (paren(cur), opc, paren(sv))
                self.wr(d, '(' + res + ')')
                if m == 'sub':
                    self.setflags('cmp', cur, sv)
                else:
                    self.setflags('res', '(' + res + ')')
            else:
                dv = self.val(d)
                self.materialize(dv)
                self.emit('%s %s= %s;' % (dv, opc, sv))
                self.setflags('res', dv)
            return
        if m in ('inc', 'dec', 'neg', 'not'):
            d = ops[0]
            if d in self.r or d in R8:
                cur = self.rd(d)
                e = {'inc': '(%s + 1)', 'dec': '(%s - 1)', 'neg': '-%s', 'not': '~%s'}[m] % paren(cur)
                self.wr(d, e)
                self.setflags('res', e)
            else:
                dv = self.val(d)
                self.materialize(dv)
                self.emit({'inc': '%s++;', 'dec': '%s--;', 'neg': '%s = -%s;', 'not': '%s = ~%s;'}[m].replace('%s', dv))
                self.setflags('res', dv)
            return
        if m in ('shl', 'sal', 'shr', 'sar', 'rol', 'ror', 'rcl', 'rcr'):
            d, s = ops if len(ops) == 2 else (ops[0], '1')
            opc = {'shl': '<<', 'sal': '<<', 'shr': '>>>', 'sar': '>>', 'rol': 'ROL', 'ror': 'ROR', 'rcl': 'RCL', 'rcr': 'RCR'}[m]
            sv = self.val(s, 8)
            if d in self.r or d in R8:
                e = '(%s %s %s)' % (paren(self.rd(d)), opc, sv)
                self.wr(d, e)
                self.setflags('res', e)
            else:
                dv = self.val(d)
                self.emit('%s %s= %s;' % (dv, opc, sv))
            return
        if m == 'cmp':
            d, s = ops
            sz = self.opsize(d, s)
            self.setflags('cmp', self.val(d, sz), self.val(s, sz))
            return
        if m == 'test':
            d, s = ops
            sz = self.opsize(d, s)
            self.setflags('test', self.val(d, sz), self.val(s, sz))
            return
        if m == 'cwde':
            m = 'cbw'
        if m == 'cdq':
            m = 'cwd'
        if m == 'cwd':
            self.wr('dx', 'SX(%s)' % self.rd('ax'))
            return
        if m == 'cbw':
            self.wr('ax', '(s8)%s' % paren(self.rd('al')))
            return
        if m in ('imul', 'mul', 'idiv', 'div'):
            if len(ops) == 1:
                s = self.val(ops[0])
                if m in ('imul', 'mul'):
                    e = '(%s * %s)' % (paren(self.rd('ax')), paren(s))
                    if ops[0] in R8 or ops[0].startswith('byte'):
                        e = '(%s * %s)' % (paren(self.rd('al')), paren(s))
                    self.wr('ax', e)
                    self.r['dx'] = 'HI(%s)' % e
                else:
                    num_ = self.rd('ax')
                    if ops[0] in R8 or ops[0].startswith('byte'):
                        self.wr('ax', '(%s / %s)' % (paren(num_), paren(s)))
                    else:
                        dxv = self.rd('dx')
                        n = num_ if dxv.startswith('SX(') or dxv == '0' else '%s:%s' % (dxv, num_)
                        self.wr('ax', '(%s / %s)' % (paren(n), paren(s)))
                        self.wr('dx', '(%s %% %s)' % (paren(n), paren(s)))
                return
            if len(ops) == 3:
                self.store(ops[0], '(%s * %s)' % (paren(self.val(ops[1])), self.val(ops[2])))
                return
            self.store(ops[0], '(%s * %s)' % (paren(self.val(ops[0])), paren(self.val(ops[1]))))
            return
        if m in JCC or m in ('js', 'jns'):
            c = self.cond(m)
            if isnum(ops[0]):
                self.flush(int(ops[0], 16))
            self.out.append('    if (%s) goto L_%04x;' % (c, int(ops[0], 16)) if isnum(ops[0]) else '    if (%s) goto %s;' % (c, ops[0]))
            return
        if m == 'jcxz':
            self.flush(-1)
            self.out.append('    if (%s == 0) goto L_%04x;' % (self.rd('cx'), int(ops[0], 16)))
            return
        if m == 'loop':
            self.flush(-1)
            self.out.append('    if (--cx) goto L_%04x;' % int(ops[0], 16))
            return
        if m == 'jmp':
            sw = self.dis.switches.get((seg, a))
            if sw:
                self.flush(-1)
                if sw[0] == 'search':
                    cases = ' '.join('%d:L_%04x' % (v if v < 0x8000 else v - 0x10000, t) for v, t in zip(sw[1], sw[2]))
                else:
                    tab, n = sw
                    cases = ' '.join('%d:L_%04x' % (k, self.dis.u16(seg, tab + k * 2)) for k in range(n))
                v = self.rd('bx')
                if v.endswith(' << 1)'):
                    v = v[1:-6]
                self.out.append('    switch (%s) { %s }' % (v if sw[0] != 'search' else '?', cases))
                return
            if isnum(ops[0]):
                self.flush(int(ops[0], 16))
            if isnum(ops[0]):
                t = int(ops[0], 16)
                self.out.append('    goto L_%04x;' % t)
            else:
                self.out.append('    goto %s;' % ops[0])
            return
        if m.startswith('rep') or m in ('movsb', 'movsw', 'stosb', 'stosw', 'scasb', 'lodsb', 'lodsw'):
            self.emit('/* %s %s */ (cx=%s, si=%s, di=%s, ax=%s)' % (m, ins.op_str, self.rd('cx'), self.rd('si'), self.rd('di'), self.rd('ax')))
            return
        if m == 'int':
            self.emit('int %s; /* ax=%s */' % (ops[0], self.rd('ax')))
            return
        if m in ('in', 'out', 'clc', 'stc', 'pushf', 'popf', 'lahf', 'sahf', 'wait', 'hlt', 'enter'):
            self.emit('/* %s %s */' % (m, ins.op_str))
            return
        self.emit('/* ?? %s %s */' % (m, ins.op_str))

    def flush_ret(self):
        ax, dx = self.r['ax'], self.r['dx']
        if 'dx' in self.dirty and dx != 'dx' and not dx.endswith('.hi'):
            self.emit('return %s:%s;' % (dx, ax))
        elif 'ax' in self.dirty and ax != 'ax':
            self.emit('return %s;' % ax)
        else:
            self.emit('return;')

    switch_var = None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe')
    ap.add_argument('funcs')
    ap.add_argument('outdir')
    ap.add_argument('--names', default=os.path.join(os.path.dirname(__file__), 're', 'names.txt'))
    ap.add_argument('--globals', default=os.path.join(os.path.dirname(__file__), 're', 'globals.txt'))
    ap.add_argument('--seg', default=None)
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
    lf = Lifter(dis, globs)
    os.makedirs(a.outdir, exist_ok=True)
    for seg in sorted(entries):
        if seg == DGROUP or seg == 0x1000:
            continue
        if a.seg and seg != int(a.seg, 16):
            continue
        offs = sorted(entries[seg])
        lines = []
        for i, o in enumerate(offs):
            if mem[seg * 16 + o] == 0xea:
                continue
            limit = offs[i + 1] if i + 1 < len(offs) else 0x10000
            lf.tcount = 0
            try:
                body = lf.lift_func(seg, o, limit)
            except Exception as e:
                body = ['// %04x:%04x LIFT ERROR %r' % (seg, o, e)]
            if body:
                lines.append('')
                lines.extend(body)
        if lines:
            open(os.path.join(a.outdir, 'seg_%04x.c' % seg), 'w').write('\n'.join(lines) + '\n')

if __name__ == '__main__':
    main()
