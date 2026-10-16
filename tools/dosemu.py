#!/usr/bin/env python3
"""Minimal DOS environment for running the original SRE.EXE under Unicorn.

Purpose: reference oracle for the reconstruction.  It runs the real
binary with a scripted remote user (through an emulated FOSSIL driver)
and records the exact byte stream the game sends to the remote side,
plus the local text screen.  Time and randomness are fully controlled.

Only the DOS/BIOS services SRE actually uses are implemented.
"""
import os, struct, sys, time, datetime, argparse
from unicorn import *
from unicorn.x86_const import *

STUB_SEG = 0xF000          # BIOS/DOS service stubs live at F000:n*4
PSP_SEG = 0x0FF0
LOAD_SEG = 0x1000
ENV_SEG = 0x0FD0
MEM_TOP = 0xA000

class DosExit(Exception):
    pass

class NeedInput(Exception):
    pass

class Dos:
    def __init__(self, root, exe, args, script=b'', clock=None, trace=False,
                 max_insns=None, ansi_out=None, idle_limit=1500):
        self.root = os.path.abspath(root)
        self.cwd = ''              # DOS-style, relative to root, no leading '\'
        self.exe = exe
        self.args = args
        self.input = bytearray(script)
        self.local_input = bytearray()
        self.snaps = None
        self.output = bytearray()
        self.ansi_out = ansi_out
        self.trace = trace
        self.files = {}
        self.next_handle = 5
        self.dta = (0, 0)
        self.find_state = None
        # virtual clock: start time (datetime) + elapsed ticks
        self.clock0 = clock or datetime.datetime(1995, 3, 8, 12, 0, 0)
        self.ticks = 0             # 18.2 Hz ticks since clock0
        self.poll_count = 0
        self.idle_polls = 0
        self.idle_limit = idle_limit
        self.exit_code = None
        self.mem_blocks = {}       # seg -> paras (allocated by 48h/4Ah)
        self.carrier = True
        self.log = []
        uc = Uc(UC_ARCH_X86, UC_MODE_16)
        uc.mem_map(0, 0x110000)
        self.uc = uc
        self._setup_ivt()
        self._load_exe()
        uc.hook_add(UC_HOOK_INTR, self._hook_intr)
        # DOSEMU_RING=n: remember the last n basic blocks, shown on a crash.
        self.ring = None
        # DOSEMU_TRACE_FROM=linear: trace instructions once that block runs.
        if os.environ.get('DOSEMU_TRACE_FROM'):
            start = int(os.environ['DOSEMU_TRACE_FROM'], 16)
            from capstone import Cs, CS_ARCH_X86, CS_MODE_16
            md = Cs(CS_ARCH_X86, CS_MODE_16)
            import collections
            self.tracebuf = collections.deque(maxlen=int(os.environ.get('DOSEMU_TRACE_KEEP', '5000')))
            want = bytes.fromhex(os.environ.get('DOSEMU_TRACE_BYTES', ''))
            state = {'on': False}
            def tr(uc, a, sz, ud):
                if a == start and not state['on'] and bytes(uc.mem_read(a, len(want))) == want:
                    state['on'] = True
                if state['on']:
                    code = bytes(uc.mem_read(a, sz))
                    ins = next(md.disasm(code, 0), None)
                    self.tracebuf.append('T %04x:%04x %s %s  ax=%04x dx=%04x bx=%04x es=%04x\n' % (
                        self.reg(UC_X86_REG_CS), self.reg(UC_X86_REG_IP),
                        ins.mnemonic if ins else '?', ins.op_str if ins else '',
                        self.reg(UC_X86_REG_AX), self.reg(UC_X86_REG_DX),
                        self.reg(UC_X86_REG_BX), self.reg(UC_X86_REG_ES)))
            uc.hook_add(UC_HOOK_BLOCK, lambda uc, a, sz, ud: tr(uc, a, sz, ud) if a == start else None)
            uc.hook_add(UC_HOOK_CODE, lambda uc, a, sz, ud: tr(uc, a, sz, ud) if state['on'] else None)
        if os.environ.get('DOSEMU_RING'):
            import collections
            self.ring = collections.deque(maxlen=int(os.environ['DOSEMU_RING']))
            uc.hook_add(UC_HOOK_BLOCK, lambda uc, a, sz, ud: self.ring.append(a))
        # Text written to the local (sysop) screen, from the entries of the
        # local stream's RawWrite (247e:0fda) and EmitColor (247e:0f8a).
        # Color changes are logged as ESC [ fg ; bg C.
        self.local_out = bytearray()
        uc.hook_add(UC_HOOK_CODE, self._hook_local, begin=0x247e0 + 0x0f8a, end=0x247e0 + 0x0f8a)
        uc.hook_add(UC_HOOK_CODE, self._hook_local, begin=0x247e0 + 0x0fda, end=0x247e0 + 0x0fda)
        uc.hook_add(UC_HOOK_CODE, self._hook_stub, begin=STUB_SEG * 16, end=STUB_SEG * 16 + 0x400)
        uc.hook_add(UC_HOOK_INSN, self._hook_in, None, 1, 0, UC_X86_INS_IN)
        uc.hook_add(UC_HOOK_INSN, self._hook_out, None, 1, 0, UC_X86_INS_OUT)

    def _hook_local(self, uc, addr, size, ud):
        if self.reg(UC_X86_REG_CS) != 0x247e:
            return
        sp = self.reg(UC_X86_REG_SS) * 16 + self.reg(UC_X86_REG_SP)
        st = bytes(uc.mem_read(sp, 14))
        this_off, this_seg = struct.unpack_from('<HH', st, 4)
        if addr == 0x247e0 + 0x0f8a:
            fg, bg = struct.unpack('<bb', bytes(uc.mem_read(this_seg * 16 + this_off + 0x1c, 2)))
            self.local_out += b'\x1b[%d;%dC' % (fg, bg)
        else:
            s_off, s_seg, n = struct.unpack_from('<HHh', st, 8)
            if n > 0:
                self.local_out += bytes(uc.mem_read(s_seg * 16 + s_off, n))

    # ------------------------------------------------------------------
    # memory helpers
    def rb(self, seg, off, n):
        return bytes(self.uc.mem_read(seg * 16 + off, n))

    def wb(self, seg, off, data):
        a = seg * 16 + off
        self.uc.mem_write(a, bytes(data))
        # Code may be loaded over code (overlays): drop stale translations.
        if len(data) >= 16:
            self.uc.ctl_remove_cache(a, a + len(data))

    def rw(self, seg, off):
        return struct.unpack('<H', self.rb(seg, off, 2))[0]

    def ww(self, seg, off, v):
        self.wb(seg, off, struct.pack('<H', v & 0xffff))

    def asciiz(self, seg, off, maxn=256):
        b = self.rb(seg, off, maxn)
        return b.split(b'\0')[0]

    def reg(self, r):
        return self.uc.reg_read(r)

    def setreg(self, r, v):
        self.uc.reg_write(r, v & 0xffff)

    # ------------------------------------------------------------------
    def _setup_ivt(self):
        for n in range(256):
            self.ww(0, n * 4, n * 4)
            self.ww(0, n * 4 + 2, STUB_SEG)
            self.wb(STUB_SEG, n * 4, b'\xcf\x90\x90\x90')   # iret (never reached)
        # BIOS data area
        self.ww(0x40, 0x10, 0x0021)      # equipment: color 80x25
        self.ww(0x40, 0x13, 640)         # memory size
        self.wb(0x40, 0x49, b'\x03')     # video mode 3
        self.ww(0x40, 0x4a, 80)          # columns
        self.ww(0x40, 0x63, 0x3d4)       # CRTC port (color)
        self.wb(0x40, 0x84, b'\x18')     # rows-1
        self.ww(0x40, 0x85, 16)          # char height
        self.ww(0x40, 0x1a, 0x1e)        # kbd buffer head
        self.ww(0x40, 0x1c, 0x1e)        # kbd buffer tail
        self.ww(0x40, 0x80, 0x1e)
        self.ww(0x40, 0x82, 0x3e)
        # clear screen
        self.wb(0xB800, 0, b'\x20\x07' * 2000)
        self.cursor = (0, 0)

    def _load_exe(self):
        d = open(self.host_path(self.exe), 'rb').read()
        (sig, lastp, pages, nrel, hdrpara, minalloc, maxalloc, ss, sp, csum,
         ip, cs, reloff, ovno) = struct.unpack('<14H', d[:28])
        imgsize = (pages - 1) * 512 + (lastp or 512)
        img = bytearray(d[hdrpara * 16:imgsize])
        for i in range(nrel):
            o, s = struct.unpack('<HH', d[reloff + i * 4:reloff + i * 4 + 4])
            a = s * 16 + o
            v = struct.unpack('<H', img[a:a + 2])[0]
            img[a:a + 2] = struct.pack('<H', (v + LOAD_SEG) & 0xffff)
        self.uc.mem_write(LOAD_SEG * 16, bytes(img))
        # environment
        env = b'COMSPEC=C:\\COMMAND.COM\0PATH=C:\\\0\0\x01\0'
        env += b'C:\\' + self.exe.upper().encode() + b'\0'
        self.wb(ENV_SEG, 0, env)
        # PSP
        psp = bytearray(256)
        psp[0:2] = b'\xcd\x20'
        struct.pack_into('<H', psp, 2, MEM_TOP)
        struct.pack_into('<H', psp, 0x2c, ENV_SEG)
        struct.pack_into('<H', psp, 0x32, 20)
        struct.pack_into('<HH', psp, 0x34, 0x18, PSP_SEG)
        psp[0x18:0x18 + 20] = bytes([1, 1, 1, 0, 2] + [0xff] * 15)
        tail = (' ' + ' '.join(self.args)).encode() if self.args else b''
        psp[0x80] = len(tail)
        psp[0x81:0x81 + len(tail)] = tail
        psp[0x81 + len(tail)] = 13
        self.wb(PSP_SEG, 0, psp)
        self.prog_paras = MEM_TOP - PSP_SEG
        uc = self.uc
        uc.reg_write(UC_X86_REG_CS, cs + LOAD_SEG)
        uc.reg_write(UC_X86_REG_IP, ip)
        uc.reg_write(UC_X86_REG_SS, ss + LOAD_SEG)
        uc.reg_write(UC_X86_REG_SP, sp)
        uc.reg_write(UC_X86_REG_DS, PSP_SEG)
        uc.reg_write(UC_X86_REG_ES, PSP_SEG)
        self.dta = (PSP_SEG, 0x80)

    # ------------------------------------------------------------------
    # path mapping (case-insensitive, rooted at self.root)
    def host_path(self, dospath, must_exist=False):
        p = dospath.replace('/', '\\')
        if len(p) >= 2 and p[1] == ':':
            p = p[2:]
        if p.startswith('\\'):
            parts = p[1:].split('\\')
        else:
            parts = [x for x in self.cwd.split('\\') if x] + p.split('\\')
        out = []
        for x in parts:
            if x in ('', '.'):
                continue
            if x == '..':
                if out:
                    out.pop()
                continue
            out.append(x)
        cur = self.root
        for i, x in enumerate(out):
            cand = None
            try:
                for e in os.listdir(cur):
                    if e.upper() == x.upper():
                        cand = e
                        break
            except OSError:
                pass
            cur = os.path.join(cur, cand if cand else x.upper())
        return cur

    # ------------------------------------------------------------------
    def now(self):
        if getattr(self, 'frozen', False):
            return self.clock0
        return self.clock0 + datetime.timedelta(seconds=self.ticks / 18.2065)

    def advance(self, ticks=1):
        self.ticks += ticks
        t = self.ticks & 0xffffffff
        self.ww(0x40, 0x6c, t & 0xffff)
        self.ww(0x40, 0x6e, (t >> 16) & 0xffff)

    def _poll(self):
        """Called when the program polls for input/time: advance the clock."""
        self.poll_count += 1
        if self.poll_count % 8 == 0:
            self.advance(1)

    # ------------------------------------------------------------------
    def _hook_in(self, uc, port, size, ud):
        if port == 0x3da:
            self._vr = getattr(self, '_vr', 0) ^ 0x09
            return self._vr
        if port == 0x40:
            # PIT channel 0: count down ~1ms per read (2 reads per sample)
            if not getattr(self, '_pithi', False):
                self._pit = (getattr(self, '_pit', 0) - 600) & 0xffff
                self._pithi = True
                return self._pit & 0xff
            self._pithi = False
            return self._pit >> 8
        if port in (0x60, 0x64):
            return 0
        return 0xff if size == 1 else 0xffff

    def _hook_out(self, uc, port, size, value, ud):
        if port == 0x43:
            self._pithi = False
        pass

    def _hook_intr(self, uc, intno, ud):
        if intno != 0x80000000 and intno < 256:
            seg = self.rw(0, intno * 4 + 2)
            off = self.rw(0, intno * 4)
            ip = self.reg(UC_X86_REG_IP)
            if seg == STUB_SEG and off == intno * 4:
                self.service(intno)
                return
            # deliver to program-installed handler
            flags = self.reg(UC_X86_REG_EFLAGS)
            sp = (self.reg(UC_X86_REG_SP) - 6) & 0xffff
            ss = self.reg(UC_X86_REG_SS)
            self.wb(ss, sp, struct.pack('<HHH', ip, self.reg(UC_X86_REG_CS), flags & 0xffff))
            self.setreg(UC_X86_REG_SP, sp)
            uc.reg_write(UC_X86_REG_EFLAGS, flags & ~0x0300)
            uc.reg_write(UC_X86_REG_CS, seg)
            uc.reg_write(UC_X86_REG_IP, off)
        else:
            raise RuntimeError('bad interrupt %r' % intno)

    def _hook_stub(self, uc, address, size, ud):
        # reached a stub via far call/jump chaining to the "original" vector
        n = (address - STUB_SEG * 16) // 4
        ss, sp = self.reg(UC_X86_REG_SS), self.reg(UC_X86_REG_SP)
        ip, cs, fl = struct.unpack('<HHH', self.rb(ss, sp, 6))
        self.setreg(UC_X86_REG_SP, sp + 6)
        uc.reg_write(UC_X86_REG_EFLAGS, fl)
        self.service(n)
        uc.reg_write(UC_X86_REG_CS, cs)
        uc.reg_write(UC_X86_REG_IP, ip)

    # ------------------------------------------------------------------
    def set_cf(self, on):
        f = self.reg(UC_X86_REG_EFLAGS)
        self.uc.reg_write(UC_X86_REG_EFLAGS, (f | 1) if on else (f & ~1))

    def set_zf(self, on):
        f = self.reg(UC_X86_REG_EFLAGS)
        self.uc.reg_write(UC_X86_REG_EFLAGS, (f | 0x40) if on else (f & ~0x40))

    def service(self, n):
        h = getattr(self, 'int_%02x' % n, None)
        if h is None:
            if self.trace:
                print('unhandled int %02x ax=%04x' % (n, self.reg(UC_X86_REG_AX)), file=sys.stderr)
            return
        h()

    # ---- interrupts ---------------------------------------------------
    def int_00(self):
        raise RuntimeError('divide error at %04x:%04x' % (self.reg(UC_X86_REG_CS), self.reg(UC_X86_REG_IP)))

    def int_08(self):
        pass

    def int_1c(self):
        pass

    def int_28(self):
        self._poll()

    def int_11(self):
        self.setreg(UC_X86_REG_AX, 0x0021)

    def int_12(self):
        self.setreg(UC_X86_REG_AX, 640)

    def int_15(self):
        ah = self.reg(UC_X86_REG_AH)
        if ah == 0x86:
            self.advance(1)
        self.set_cf(True)
        self.uc.reg_write(UC_X86_REG_AH, 0x86)

    def int_1a(self):
        ah = self.reg(UC_X86_REG_AH)
        self._poll()
        if ah == 0:
            t = self.ticks
            self.setreg(UC_X86_REG_CX, t >> 16)
            self.setreg(UC_X86_REG_DX, t & 0xffff)
            self.uc.reg_write(UC_X86_REG_AL, 0)
        elif ah == 2:
            n = self.now()
            bcd = lambda v: ((v // 10) << 4) | (v % 10)
            self.uc.reg_write(UC_X86_REG_CH, bcd(n.hour))
            self.uc.reg_write(UC_X86_REG_CL, bcd(n.minute))
            self.uc.reg_write(UC_X86_REG_DH, bcd(n.second))
            self.set_cf(False)
        elif ah == 4:
            n = self.now()
            bcd = lambda v: ((v // 10) << 4) | (v % 10)
            self.uc.reg_write(UC_X86_REG_CH, bcd(n.year // 100))
            self.uc.reg_write(UC_X86_REG_CL, bcd(n.year % 100))
            self.uc.reg_write(UC_X86_REG_DH, bcd(n.month))
            self.uc.reg_write(UC_X86_REG_DL, bcd(n.day))
            self.set_cf(False)

    def int_2f(self):
        ax = self.reg(UC_X86_REG_AX)
        if ax == 0x1680:          # release time slice
            self._poll()
            self.uc.reg_write(UC_X86_REG_AL, 0x80)
        # everything else: not installed (AL unchanged = 0)

    def int_67(self):
        self.uc.reg_write(UC_X86_REG_AH, 0x80)

    def int_33(self):
        self.setreg(UC_X86_REG_AX, 0)

    def int_10(self):
        ah = self.reg(UC_X86_REG_AH)
        if ah == 0x0f:
            self.uc.reg_write(UC_X86_REG_AL, 3)
            self.uc.reg_write(UC_X86_REG_AH, 80)
            self.uc.reg_write(UC_X86_REG_BH, 0)
        elif ah == 0x03:
            r, c = self.cursor
            self.uc.reg_write(UC_X86_REG_DH, r)
            self.uc.reg_write(UC_X86_REG_DL, c)
            self.setreg(UC_X86_REG_CX, 0x0607)
        elif ah == 0x02:
            self.cursor = (self.reg(UC_X86_REG_DH), self.reg(UC_X86_REG_DL))
        elif ah == 0x08:
            r, c = self.cursor
            ch, at = self.rb(0xB800, (r * 80 + c) * 2, 2)
            self.uc.reg_write(UC_X86_REG_AL, ch)
            self.uc.reg_write(UC_X86_REG_AH, at)
        elif ah in (0x09, 0x0a):
            r, c = self.cursor
            ch = self.reg(UC_X86_REG_AL)
            at = self.reg(UC_X86_REG_BL)
            cnt = self.reg(UC_X86_REG_CX)
            for i in range(cnt):
                o = (r * 80 + c + i)
                if o >= 2000:
                    break
                if ah == 0x09:
                    self.wb(0xB800, o * 2, bytes([ch, at]))
                else:
                    self.wb(0xB800, o * 2, bytes([ch]))
        elif ah == 0x0e:
            self._tty(self.reg(UC_X86_REG_AL))
        elif ah in (0x06, 0x07):
            al = self.reg(UC_X86_REG_AL)
            bh = self.reg(UC_X86_REG_BH)
            r0, c0 = self.reg(UC_X86_REG_CH), self.reg(UC_X86_REG_CL)
            r1, c1 = self.reg(UC_X86_REG_DH), self.reg(UC_X86_REG_DL)
            self._scroll(r0, c0, r1, c1, al, bh, ah == 0x06)
        elif ah == 0x1a:
            self.uc.reg_write(UC_X86_REG_AL, 0x1a)
            self.setreg(UC_X86_REG_BX, 0x0008)
        elif ah == 0x12:
            self.setreg(UC_X86_REG_BX, 0x0003)
            self.setreg(UC_X86_REG_CX, 0x0009)
        elif ah == 0x01 or ah == 0x10 or ah == 0x00 or ah == 0x05 or ah == 0x0b:
            pass
        else:
            if self.trace:
                print('int10 ah=%02x' % ah, file=sys.stderr)

    def _scroll(self, r0, c0, r1, c1, n, attr, up):
        rows = r1 - r0 + 1
        if n == 0 or n > rows:
            n = rows
        scr = bytearray(self.rb(0xB800, 0, 4000))
        def cell(r, c):
            return (r * 80 + c) * 2
        rng = range(r0, r1 + 1) if up else range(r1, r0 - 1, -1)
        for r in rng:
            src = r + n if up else r - n
            for c in range(c0, c1 + 1):
                if r0 <= src <= r1:
                    scr[cell(r, c):cell(r, c) + 2] = scr[cell(src, c):cell(src, c) + 2]
                else:
                    scr[cell(r, c):cell(r, c) + 2] = bytes([32, attr])
        self.wb(0xB800, 0, scr)

    def _tty(self, ch):
        r, c = self.cursor
        if ch == 13:
            c = 0
        elif ch == 10:
            r += 1
        elif ch == 8:
            c = max(0, c - 1)
        elif ch == 7:
            pass
        else:
            self.wb(0xB800, (r * 80 + c) * 2, bytes([ch]))
            c += 1
            if c >= 80:
                c = 0
                r += 1
        if r >= 25:
            self._scroll(0, 0, 24, 79, 1, 7, True)
            r = 24
        self.cursor = (r, c)

    def _localkey(self):
        c = self.local_input[0]
        if c == 0:                 # \0 + scancode = extended key
            return self.local_input[1] << 8
        return c

    def int_16(self):
        ah = self.reg(UC_X86_REG_AH)
        self._poll()
        if ah in (0x01, 0x11):
            if self.local_input:
                self.set_zf(False)
                self.setreg(UC_X86_REG_AX, self._localkey())
            else:
                self.set_zf(True)
                self.setreg(UC_X86_REG_AX, 0)
                self._idle()
        elif ah in (0x00, 0x10):
            if not self.local_input:
                raise NeedInput()
            self._snap()
            self.setreg(UC_X86_REG_AX, self._localkey())
            del self.local_input[:2 if self.local_input[0] == 0 else 1]
        elif ah in (0x02, 0x12):
            self.uc.reg_write(UC_X86_REG_AL, 0)

    # ---- FOSSIL --------------------------------------------------------
    def int_14(self):
        ah = self.reg(UC_X86_REG_AH)
        if ah == 0x04 or ah == 0x1c:
            self.setreg(UC_X86_REG_AX, 0x1954)
            self.uc.reg_write(UC_X86_REG_BL, 0x1b)
            self.uc.reg_write(UC_X86_REG_BH, 5)
        elif ah == 0x05 or ah == 0x1d:
            pass
        elif ah == 0x00:
            self.setreg(UC_X86_REG_AX, self._fstatus())
        elif ah == 0x01 or ah == 0x0b:
            self._send(bytes([self.reg(UC_X86_REG_AL)]))
            self.setreg(UC_X86_REG_AX, self._fstatus() if ah == 1 else 1)
        elif ah == 0x02:
            self._poll()
            if self.input and self.input[0] == 0:
                del self.input[0]
            if not self.input:
                raise NeedInput()
            self._snap()
            self.setreg(UC_X86_REG_AX, self.input.pop(0))
        elif ah == 0x03:
            self._poll()
            self.setreg(UC_X86_REG_AX, self._fstatus())
            if not self._avail():
                self._idle()
        elif ah == 0x0c:
            self._poll()
            self.setreg(UC_X86_REG_AX, self.input[0] if self._avail() else 0xffff)
            if not self._avail():
                self._idle()
        elif ah == 0x0d:
            self.setreg(UC_X86_REG_AX, 0xffff)
        elif ah == 0x0e:
            self._send(bytes([self.reg(UC_X86_REG_AL)]))
        elif ah == 0x06 or ah == 0x08 or ah == 0x09 or ah == 0x0a or ah == 0x0f or ah == 0x10 or ah == 0x1a:
            if ah == 0x0a:
                pass
            pass
        elif ah == 0x18:
            cx = self.reg(UC_X86_REG_CX)
            n = min(cx, len(self.input))
            if 0 in self.input[:n]:
                n = self.input.index(0)
            es, di = self.reg(UC_X86_REG_ES), self.reg(UC_X86_REG_DI)
            self.wb(es, di, bytes(self.input[:n]))
            del self.input[:n]
            self.setreg(UC_X86_REG_AX, n)
        elif ah == 0x19:
            cx = self.reg(UC_X86_REG_CX)
            es, di = self.reg(UC_X86_REG_ES), self.reg(UC_X86_REG_DI)
            self._send(self.rb(es, di, cx))
            self.setreg(UC_X86_REG_AX, cx)
        elif ah == 0x1b:
            cx = self.reg(UC_X86_REG_CX)
            es, di = self.reg(UC_X86_REG_ES), self.reg(UC_X86_REG_DI)
            info = struct.pack('<HBBHHHHHHBBBB', 19, 5, 0, 0, 0, 1024, 1024 - len(self.input),
                               1024, 1024, 80, 25, 0x23, 0)[:cx]
            self.wb(es, di, info)
            self.setreg(UC_X86_REG_AX, len(info))
        else:
            if self.trace:
                print('fossil ah=%02x' % ah, file=sys.stderr)

    def _snap(self):
        if self.snaps is not None:
            t = self.screen_text()
            if not self.snaps or self.snaps[-1] != t:
                self.snaps.append(t)

    def _fstatus(self):
        ah = 0x60 | (1 if self._avail() else 0)
        al = 0x08 | (0x80 if self.carrier else 0)
        return (ah << 8) | al

    def _send(self, b):
        self.output.extend(b)
        self.idle_polls = 0
        if self.ansi_out:
            self.ansi_out.write(b)
            self.ansi_out.flush()

    # A NUL in the input script holds back what follows until the program
    # sits waiting for a key (so key-ready checks can't swallow it).
    def _avail(self):
        return bool(self.input) and self.input[0] != 0

    def _idle(self):
        if self._avail() or self.local_input:
            self.idle_polls = 0
            return
        self.idle_polls += 1
        if self.input and self.idle_polls > 500:
            del self.input[0]
            self.idle_polls = 0
            return
        if self.idle_polls > self.idle_limit:
            raise NeedInput()

    # ---- DOS -----------------------------------------------------------
    def int_20(self):
        raise DosExit(0)

    def int_21(self):
        ah = self.reg(UC_X86_REG_AH)
        fn = getattr(self, 'dos_%02x' % ah, None)
        if fn is None:
            if self.trace:
                print('unhandled dos %02x ax=%04x at %04x:%04x' % (
                    ah, self.reg(UC_X86_REG_AX), self.reg(UC_X86_REG_CS), self.reg(UC_X86_REG_IP)),
                    file=sys.stderr)
            self.set_cf(True)
            self.setreg(UC_X86_REG_AX, 1)
            return
        fn()

    def ok(self, ax=None):
        if ax is not None:
            self.setreg(UC_X86_REG_AX, ax)
        self.set_cf(False)

    def err(self, code):
        self.setreg(UC_X86_REG_AX, code)
        self.set_cf(True)

    def dos_02(self):
        self._tty(self.reg(UC_X86_REG_DL))

    def dos_06(self):
        dl = self.reg(UC_X86_REG_DL)
        if dl != 0xff:
            self._tty(dl)
        else:
            self._poll()
            self.set_zf(True)
            self.uc.reg_write(UC_X86_REG_AL, 0)

    def dos_09(self):
        ds, dx = self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_DX)
        s = self.rb(ds, dx, 2000).split(b'$')[0]
        for c in s:
            self._tty(c)

    def dos_0b(self):           # kbhit(): the local keyboard
        self._poll()
        if self.local_input:
            self.uc.reg_write(UC_X86_REG_AL, 0xff)
        else:
            self.uc.reg_write(UC_X86_REG_AL, 0)
            self._idle()

    def dos_0e(self):
        self.uc.reg_write(UC_X86_REG_AL, 3)

    def dos_19(self):
        self.uc.reg_write(UC_X86_REG_AL, 2)

    def dos_1a(self):
        self.dta = (self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_DX))

    def dos_25(self):
        n = self.reg(UC_X86_REG_AL)
        self.ww(0, n * 4, self.reg(UC_X86_REG_DX))
        self.ww(0, n * 4 + 2, self.reg(UC_X86_REG_DS))

    def dos_2a(self):
        n = self.now()
        self.setreg(UC_X86_REG_CX, n.year)
        self.uc.reg_write(UC_X86_REG_DH, n.month)
        self.uc.reg_write(UC_X86_REG_DL, n.day)
        self.uc.reg_write(UC_X86_REG_AL, (n.weekday() + 1) % 7)

    def dos_2c(self):
        self._poll()
        n = self.now()
        self.uc.reg_write(UC_X86_REG_CH, n.hour)
        self.uc.reg_write(UC_X86_REG_CL, n.minute)
        self.uc.reg_write(UC_X86_REG_DH, n.second)
        self.uc.reg_write(UC_X86_REG_DL, n.microsecond // 10000)

    def dos_2b(self):
        self.uc.reg_write(UC_X86_REG_AL, 0xff)

    def dos_2d(self):
        self.uc.reg_write(UC_X86_REG_AL, 0xff)

    def dos_2f(self):
        self.setreg(UC_X86_REG_ES, self.dta[0])
        self.setreg(UC_X86_REG_BX, self.dta[1])

    def dos_30(self):
        self.setreg(UC_X86_REG_AX, 0x0005)    # DOS 5.0
        self.setreg(UC_X86_REG_BX, 0)
        self.setreg(UC_X86_REG_CX, 0)

    def dos_33(self):
        self.uc.reg_write(UC_X86_REG_DL, 0)

    def dos_35(self):
        n = self.reg(UC_X86_REG_AL)
        self.setreg(UC_X86_REG_BX, self.rw(0, n * 4))
        self.setreg(UC_X86_REG_ES, self.rw(0, n * 4 + 2))

    def dos_36(self):
        self.setreg(UC_X86_REG_AX, 64)
        self.setreg(UC_X86_REG_BX, 0x7fff)
        self.setreg(UC_X86_REG_CX, 512)
        self.setreg(UC_X86_REG_DX, 0xffff)

    def dos_44(self):
        al = self.reg(UC_X86_REG_AL)
        bx = self.reg(UC_X86_REG_BX)
        if al == 0:
            if bx < 5:
                self.ok(0x80d3 if bx != 4 else 0x80c0)
                self.setreg(UC_X86_REG_DX, 0x80d3 if bx != 4 else 0x80c0)
            elif bx in self.files:
                self.setreg(UC_X86_REG_DX, 0x0002)
                self.ok(0x0002)
            else:
                self.err(6)
        elif al == 1:
            self.ok()
        elif al == 8:
            self.ok(1)
        elif al == 9:
            self.setreg(UC_X86_REG_DX, 0)
            self.ok()
        else:
            self.err(1)

    def _path(self):
        return self.asciiz(self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_DX)).decode('cp437')

    def _newhandle(self, f, name):
        h = self.next_handle
        while h in self.files:
            h += 1
        self.files[h] = (f, name)
        return h

    def dos_3c(self, excl=False):
        name = self._path()
        hp = self.host_path(name)
        if excl and os.path.exists(hp):
            return self.err(80)
        try:
            os.makedirs(os.path.dirname(hp), exist_ok=True) if False else None
            f = open(hp, 'w+b')
        except OSError:
            return self.err(3)
        self.log.append(('create', name))
        self.ok(self._newhandle(f, name))

    def dos_5b(self):
        self.dos_3c(excl=True)

    def dos_3d(self):
        name = self._path()
        mode = self.reg(UC_X86_REG_AL) & 7
        hp = self.host_path(name)
        if self.trace:
            print('open %s mode %d -> %s' % (name, mode, os.path.exists(hp)), file=sys.stderr)
        if not os.path.isfile(hp):
            return self.err(2)
        try:
            f = open(hp, 'rb' if mode == 0 else 'r+b')
        except OSError:
            return self.err(5)
        self.log.append(('open', name))
        self.ok(self._newhandle(f, name))

    def dos_3e(self):
        bx = self.reg(UC_X86_REG_BX)
        if bx in self.files:
            self.files.pop(bx)[0].close()
            self.ok()
        elif bx < 5:
            self.ok()
        else:
            self.err(6)

    def dos_3f(self):
        bx, cx = self.reg(UC_X86_REG_BX), self.reg(UC_X86_REG_CX)
        ds, dx = self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_DX)
        if bx in self.files:
            data = self.files[bx][0].read(cx)
            self.wb(ds, dx, data)
            self.ok(len(data))
        elif bx == 0:
            self.ok(0)
        else:
            self.err(6)

    def dos_40(self):
        bx, cx = self.reg(UC_X86_REG_BX), self.reg(UC_X86_REG_CX)
        ds, dx = self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_DX)
        data = self.rb(ds, dx, cx)
        if bx in self.files:
            f = self.files[bx][0]
            if cx == 0:
                f.truncate()
            else:
                f.write(data)
            self.ok(cx)
        elif bx in (1, 2):
            for c in data:
                self._tty(c)
            self.ok(cx)
        elif bx in (3, 4):
            self.ok(cx)
        else:
            self.err(6)

    def dos_41(self):
        hp = self.host_path(self._path())
        try:
            os.remove(hp)
            self.ok()
        except OSError:
            self.err(2)

    def dos_42(self):
        bx = self.reg(UC_X86_REG_BX)
        al = self.reg(UC_X86_REG_AL)
        off = (self.reg(UC_X86_REG_CX) << 16) | self.reg(UC_X86_REG_DX)
        if off & 0x80000000:
            off -= 1 << 32
        if bx not in self.files:
            if bx < 5:
                self.setreg(UC_X86_REG_DX, 0)
                return self.ok(0)
            return self.err(6)
        f = self.files[bx][0]
        f.seek(off, al)
        p = f.tell()
        self.setreg(UC_X86_REG_DX, p >> 16)
        self.ok(p & 0xffff)

    def dos_43(self):
        hp = self.host_path(self._path())
        if not os.path.exists(hp):
            return self.err(2)
        if self.reg(UC_X86_REG_AL) == 0:
            self.setreg(UC_X86_REG_CX, 0x10 if os.path.isdir(hp) else 0x20)
        self.ok()

    def dos_39(self):
        hp = self.host_path(self._path())
        try:
            os.mkdir(hp)
            self.ok()
        except OSError:
            self.err(5)

    def dos_3a(self):
        try:
            os.rmdir(self.host_path(self._path()))
            self.ok()
        except OSError:
            self.err(5)

    def dos_3b(self):
        p = self._path()
        hp = self.host_path(p)
        if not os.path.isdir(hp):
            return self.err(3)
        rel = os.path.relpath(hp, self.root)
        self.cwd = '' if rel == '.' else rel.replace('/', '\\')
        self.ok()

    def dos_45(self):
        bx = self.reg(UC_X86_REG_BX)
        if bx in self.files:
            f, name = self.files[bx]
            nf = open(f.name, f.mode)
            nf.seek(f.tell())
            self.ok(self._newhandle(nf, name))
        else:
            self.ok(bx)

    def dos_46(self):
        self.ok()

    def dos_47(self):
        ds, si = self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_SI)
        self.wb(ds, si, self.cwd.upper().encode() + b'\0')
        self.ok()

    def dos_48(self):
        bx = self.reg(UC_X86_REG_BX)
        # allocate from the top of the free area above the program block
        used_top = PSP_SEG + self.prog_paras
        segs = sorted(self.mem_blocks.items())
        start = used_top
        for s, n in segs:
            start = max(start, s + n)
        avail = MEM_TOP - start
        if bx > avail:
            self.setreg(UC_X86_REG_BX, max(0, avail - 1))
            return self.err(8)
        self.mem_blocks[start + 1] = bx
        self.ok(start + 1)

    def dos_49(self):
        es = self.reg(UC_X86_REG_ES)
        self.mem_blocks.pop(es, None)
        self.ok()

    def dos_4a(self):
        es, bx = self.reg(UC_X86_REG_ES), self.reg(UC_X86_REG_BX)
        if es == PSP_SEG:
            limit = MEM_TOP - PSP_SEG
            for s, n in self.mem_blocks.items():
                limit = min(limit, s - 1 - PSP_SEG)
            if bx > limit:
                self.setreg(UC_X86_REG_BX, limit)
                return self.err(8)
            self.prog_paras = bx
            return self.ok()
        if es in self.mem_blocks:
            self.mem_blocks[es] = bx
            return self.ok()
        self.err(9)

    def dos_4b(self):
        self.err(2)

    def dos_4c(self):
        self.exit_code = self.reg(UC_X86_REG_AL)
        raise DosExit(self.exit_code)

    def dos_4e(self):
        spec = self._path()
        attr = self.reg(UC_X86_REG_CX)
        d, _, pat = spec.replace('/', '\\').rpartition('\\')
        hd = self.host_path(d if d else '.')
        import fnmatch
        try:
            names = sorted(os.listdir(hd))
        except OSError:
            names = []
        p = pat.upper()
        if p.endswith('.*'):
            pass
        matches = []
        for nme in names:
            u = nme.upper()
            full = os.path.join(hd, nme)
            isdir = os.path.isdir(full)
            if isdir and not (attr & 0x10):
                continue
            if fnmatch.fnmatchcase(u, p) or (p.endswith('.*') and '.' not in u and fnmatch.fnmatchcase(u, p[:-2])):
                matches.append((u, full, isdir))
        self.find_state = matches
        self.dos_4f()

    def dos_4f(self):
        if not self.find_state:
            return self.err(18)
        u, full, isdir = self.find_state.pop(0)
        st = os.stat(full)
        t = datetime.datetime.fromtimestamp(st.st_mtime)
        ftime = (t.hour << 11) | (t.minute << 5) | (t.second // 2)
        fdate = ((t.year - 1980) << 9) | (t.month << 5) | t.day
        rec = bytearray(43)
        rec[21] = 0x10 if isdir else 0x20
        struct.pack_into('<HHI', rec, 22, ftime, fdate, st.st_size if not isdir else 0)
        nm = u.encode()[:12]
        rec[30:30 + len(nm)] = nm
        self.wb(self.dta[0], self.dta[1], rec)
        self.ok(0)

    def dos_56(self):
        old = self.host_path(self._path())
        es, di = self.reg(UC_X86_REG_ES), self.reg(UC_X86_REG_DI)
        new = self.host_path(self.asciiz(es, di).decode('cp437'))
        try:
            os.rename(old, new)
            self.ok()
        except OSError:
            self.err(5)

    def dos_57(self):
        bx = self.reg(UC_X86_REG_BX)
        if self.reg(UC_X86_REG_AL) == 0:
            n = self.now()
            self.setreg(UC_X86_REG_CX, (n.hour << 11) | (n.minute << 5) | (n.second // 2))
            self.setreg(UC_X86_REG_DX, ((n.year - 1980) << 9) | (n.month << 5) | n.day)
        self.ok()

    def dos_37(self):
        if self.reg(UC_X86_REG_AL) == 0:
            self.uc.reg_write(UC_X86_REG_DL, ord('/'))
        self.uc.reg_write(UC_X86_REG_AL, 0)

    def dos_38(self):
        # US country info
        info = bytearray(34)
        struct.pack_into('<H', info, 0, 0)          # date format m-d-y
        info[2:4] = b'$\0'
        info[7:9] = b',\0'
        info[9:11] = b'.\0'
        info[11:13] = b'-\0'
        info[13:15] = b':\0'
        info[15] = 0
        info[16] = 2
        info[17] = 0
        info[22:24] = b',\0'
        self.wb(self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_DX), info)
        self.setreg(UC_X86_REG_BX, 1)
        self.ok(1)

    def dos_58(self):
        self.ok(0)

    def dos_59(self):
        self.setreg(UC_X86_REG_AX, 2)
        self.setreg(UC_X86_REG_BX, 0x0201)
        self.setreg(UC_X86_REG_CX, 0)

    def dos_5a(self):
        ds, dx = self.reg(UC_X86_REG_DS), self.reg(UC_X86_REG_DX)
        base = self.asciiz(ds, dx).decode('cp437')
        i = 0
        while True:
            nm = 'TMP%05d.$$$' % i
            if not os.path.exists(self.host_path(base + nm)):
                break
            i += 1
        self.wb(ds, dx + len(base), nm.encode() + b'\0')
        f = open(self.host_path(base + nm), 'w+b')
        self.ok(self._newhandle(f, base + nm))

    def dos_62(self):
        self.setreg(UC_X86_REG_BX, PSP_SEG)

    def dos_63(self):
        self.err(1)

    def dos_67(self):
        self.ok()

    def dos_68(self):
        self.ok()

    def dos_0d(self):
        pass

    # ------------------------------------------------------------------
    def screen_text(self):
        scr = self.rb(0xB800, 0, 4000)
        lines = []
        for r in range(25):
            lines.append(bytes(scr[(r * 80 + c) * 2] for c in range(80)).decode('cp437').rstrip())
        return '\n'.join(lines)

    def run(self, max_steps=None, chunk=2000000):
        uc = self.uc
        done = 0
        while True:
            cs = self.reg(UC_X86_REG_CS)
            ip = self.reg(UC_X86_REG_IP)
            try:
                uc.emu_start(cs * 16 + ip, 0x200000, count=chunk)
                done += chunk
                # a chunk ended without an exit/input request: the program
                # is computing or busy-waiting; let time pass.
                self.advance(1)
                if max_steps and done >= max_steps:
                    return 'stopped'
                continue
            except DosExit as e:
                return 'exit'
            except NeedInput:
                return 'input'
            except UcError as e:
                cs = self.reg(UC_X86_REG_CS)
                ip = self.reg(UC_X86_REG_IP)
                if self.ring:
                    for a in self.ring:
                        sys.stderr.write('  block %05x\n' % a)
                if getattr(self, 'tracebuf', None):
                    sys.stderr.write(''.join(self.tracebuf))
                if os.environ.get('DOSEMU_CRASHDUMP'):
                    open(os.environ['DOSEMU_CRASHDUMP'], 'wb').write(bytes(self.uc.mem_read(0, 0x100000)))
                raise RuntimeError('%s at %04x:%04x' % (e, cs, ip))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('root')
    ap.add_argument('args', nargs='*')
    ap.add_argument('--exe', default='SRE.EXE')
    ap.add_argument('--input', default='')
    ap.add_argument('--input-file')
    ap.add_argument('--local', default='')
    ap.add_argument('--trace', action='store_true')
    ap.add_argument('--out')
    ap.add_argument('--local-out', help='write the local screen text here')
    ap.add_argument('--screen', action='store_true')
    ap.add_argument('--steps', type=int, default=0)
    ap.add_argument('--snaps', action='store_true')
    ap.add_argument('--clock', default='1995-03-08T12:00:00')
    ap.add_argument('--frozen', action='store_true', help='date/time services always return --clock')
    ap.add_argument('--dump-ptr', action='append', default=[],
                    help='OFF:LEN:FILE - at exit, write LEN bytes at the far pointer stored at DS:OFF')
    a = ap.parse_args()
    script = a.input.encode('latin-1').decode('unicode_escape').encode('latin-1')
    if a.input_file:
        script += open(a.input_file, 'rb').read()
    clock = datetime.datetime.fromisoformat(a.clock)
    dos = Dos(a.root, a.exe, a.args, script, clock=clock, trace=a.trace)
    dos.frozen = a.frozen
    if a.snaps:
        dos.snaps = []
    dos.local_input = bytearray(a.local.encode('latin-1').decode('unicode_escape').encode('latin-1'))
    r = dos.run(max_steps=a.steps)
    if getattr(dos, "tracebuf", None):
        sys.stderr.write("".join(dos.tracebuf))
    if r == 'stopped':
        print('stopped at %04x:%04x' % (dos.reg(UC_X86_REG_CS), dos.reg(UC_X86_REG_IP)), file=sys.stderr)
    if a.local_out:
        open(a.local_out, 'wb').write(bytes(dos.local_out))
    if a.out:
        open(a.out, 'wb').write(bytes(dos.output))
    else:
        sys.stdout.buffer.write(bytes(dos.output))
    print('\n[result=%s exit=%r input_left=%d]' % (r, dos.exit_code, len(dos.input)), file=sys.stderr)
    for spec in a.dump_ptr:
        off, n, fn = spec.split(':', 2)
        ds = dos.reg(UC_X86_REG_DS)
        p = dos.uc.mem_read(ds * 16 + int(off, 16), 4)
        poff = p[0] | p[1] << 8
        pseg = p[2] | p[3] << 8
        open(fn, 'wb').write(bytes(dos.uc.mem_read(pseg * 16 + poff, int(n, 16))))
    if a.snaps:
        for t in dos.snaps:
            print('=' * 80, file=sys.stderr)
            print(t, file=sys.stderr)
    if a.screen:
        print('=' * 80, file=sys.stderr)
        print(dos.screen_text(), file=sys.stderr)

if __name__ == '__main__':
    main()
