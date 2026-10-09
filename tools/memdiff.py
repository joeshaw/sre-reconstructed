#!/usr/bin/env python3
"""Run the original (emulated) and the port on copies of a game with the
same input, dump the empire arrays when the input runs out, and list the
fields that differ.

usage: tools/memdiff.py TEMPLATE WORK 'input' [--clock ISO]"""
import os, shutil, struct, subprocess, sys, calendar, datetime
here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
t, w, inp = sys.argv[1:4]
clock = sys.argv[sys.argv.index('--clock') + 1] if '--clock' in sys.argv else '1995-03-08T12:00:00'
shutil.rmtree(w, ignore_errors=True)
os.makedirs(w)
shutil.copytree(t, w + '/orig'); shutil.copytree(t, w + '/port')
subprocess.run([sys.executable, '-I', here + '/tools/dosemu.py', w + '/orig', '--input', inp, '--frozen',
                '--clock', clock, '--out', w + '/orig.out', '--dump-ptr', 'c92e:4a9c:' + w + '/orig.emp'],
               stderr=subprocess.DEVNULL)
secs = calendar.timegm(datetime.datetime.fromisoformat(clock).timetuple())
env = dict(os.environ, SRE_TESTMARK='1', SRE_UNREGISTERED='1', SRE_CLOCK=str(secs), SRE_DUMP=w + '/port.emp')
data = inp.encode('latin-1').decode('unicode_escape').encode('latin-1')
subprocess.run([here + '/sre'], input=data, cwd=w + '/port', env=env, stdout=open(w + '/port.out', 'wb'))
a = open(w + '/orig.emp', 'rb').read(); b = open(w + '/port.emp', 'rb').read()
for e in range(25):
    x = a[e * 0x2fc:(e + 1) * 0x2fc]; y = b[e * 0x2fc:(e + 1) * 0x2fc]
    if x == y:
        continue
    for off in range(0, 0x2fc):
        if x[off] != y[off]:
            print('empire %d +%03x: orig %02x port %02x' % (e, off, x[off], y[off]))
