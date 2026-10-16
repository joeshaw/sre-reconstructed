#!/usr/bin/env python3
"""Compare the original's local screen log (dosemu --local-out, color
changes as ESC [ fg ; bg C) with the reconstruction's terminal output
(ANSI SGR colors).  Both are reduced to characters with their colors."""
import re, sys

ANSI = [0, 4, 2, 6, 1, 5, 3, 7]   # SGR color number -> PC color


def chars(data, sgr):
    fg, bg = 7, 0
    out = []
    i = 0
    tok = re.compile(rb'\x1b\[([0-9;-]*)([A-Za-z])')
    while i < len(data):
        m = tok.match(data, i)
        if m:
            args, cmd = m.group(1), m.group(2)
            if sgr and cmd == b'm':
                for a in [int(x) for x in args.split(b';') if x] or [0]:
                    if a == 0:
                        fg, bg = 7, 0
                    elif a == 1:
                        fg |= 8
                    elif 30 <= a <= 37:
                        fg = (fg & 8) | ANSI[a - 30]
                    elif 40 <= a <= 47:
                        bg = ANSI[a - 40]
            elif not sgr and cmd == b'C':
                f, b = [int(x) for x in args.split(b';')]
                fg = f
                if b != -1:
                    bg = b
            else:
                out.extend((c, fg, bg) for c in m.group(0))
            i = m.end()
            continue
        out.append((data[i], fg, bg))
        i += 1
    return out


def show(seq):
    return bytes(c for c, f, b in seq).decode('cp437')


def main():
    a = chars(open(sys.argv[1], 'rb').read(), False)
    # Drop the reconstruction's conio output (bracketed in test runs).
    port = re.sub(rb'\x1bP.*?\x1b\\', b'', open(sys.argv[2], 'rb').read(), flags=re.S)
    b = chars(port, True)
    textdiff = None
    for i in range(min(len(a), len(b))):
        if a[i][0] != b[i][0]:
            textdiff = i
            break
    else:
        if len(a) != len(b):
            textdiff = min(len(a), len(b))
    if textdiff is not None:
        i = textdiff
        print('local text differs at char %d (orig %d, port %d chars)' % (i, len(a), len(b)))
        print(' orig: %r' % show(a[max(0, i - 60):i + 60]))
        print(' port: %r' % show(b[max(0, i - 60):i + 60]))
        return 1
    for i in range(len(a)):
        # Colors of spaces and line ends are invisible.
        if a[i][0] > 32 and a[i][1:] != b[i][1:]:
            print('local color differs at char %d %r: orig %s, port %s' % (
                i, show(a[max(0, i - 30):i + 1]), a[i][1:], b[i][1:]))
            return 1
    print('local: same (%d chars)' % len(a))
    return 0


sys.exit(main())
