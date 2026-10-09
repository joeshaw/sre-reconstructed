#!/usr/bin/env python3
"""Render terminal output (CP437 bytes with ANSI sequences) on an 80x25
screen and print the final screen, to compare with dosemu --screen."""
import sys
import pyte

screen = pyte.Screen(80, 25)
stream = pyte.ByteStream(screen)
data = open(sys.argv[1], 'rb').read().replace(b'\x1bP', b'').replace(b'\x1b\\', b'')
# pyte decodes UTF-8: re-encode the CP437 text first.
stream.feed(data.decode('cp437').replace('\r\n', '\r\n').encode('utf-8'))
for line in screen.display:
    print(line.rstrip())
