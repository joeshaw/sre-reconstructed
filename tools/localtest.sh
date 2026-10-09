#!/bin/bash
# Differential test of the local (sysop) functions: run the original with
# local keyboard input and the reconstruction with the same keys on its
# terminal, then compare the local screen text and the data.
#
# usage: tools/localtest.sh TEMPLATE_DIR WORK_DIR 'keys' [args...]
# Keys are printf %b escapes; {up} {down} {left} {right} {home} {end}
# {pgup} {pgdn} are the cursor keys.
set -u
T=$1; W=$2; KEYS=$3; shift 3
keys() {
    python3 - "$1" "$KEYS" <<'PY'
import sys
dos = sys.argv[1] == 'dos'
codes = {'up': (0x48, 'A'), 'down': (0x50, 'B'), 'right': (0x4d, 'C'), 'left': (0x4b, 'D'),
         'home': (0x47, 'H'), 'end': (0x4f, 'F'), 'pgup': (0x49, '5~'), 'pgdn': (0x51, '6~')}
s = sys.argv[2]
for k, (scan, seq) in codes.items():
    s = s.replace('{%s}' % k, '\\x00\\x%02x' % scan if dos else '\\x1b[' + seq)
print(s, end='')
PY
}
IN=$(keys dos); PIN=$(keys port)
HERE=$(cd "$(dirname "$0")/.." && pwd)
CLOCK=${CLOCK:-1995-03-08T12:00:00}
SECS=$(python3 -c "import datetime,calendar;print(calendar.timegm(datetime.datetime.fromisoformat('$CLOCK').timetuple()))")
rm -rf "$W"; mkdir -p "$W"
cp -r "$T" "$W/orig"; cp -r "$T" "$W/port"
timeout 900 python3 -I "$HERE/tools/dosemu.py" "$W/orig" "$@" --local "$IN" --frozen --clock "$CLOCK" \
    --local-out "$W/orig.out" --screen 2> "$W/orig.err"
printf '%b' "$PIN" | (cd "$W/port" && SRE_TESTMARK=1 SRE_UNREGISTERED=1 SRE_CLOCK=$SECS timeout 60 "$HERE/sre" "$@" > "$W/port.out")
echo "port exit=$?"
grep -a '^\[result' "$W/orig.err"
status=0
python3 "$HERE/tools/localdiff.py" "$W/orig.out" "$W/port.out" || status=1
python3 "$HERE/tools/sredata.py" "$W/orig" --dump "$W/orig.data" > "$W/orig.salts" 2>&1
python3 "$HERE/tools/sredata.py" "$W/port" --dump "$W/port.data" > "$W/port.salts" 2>&1
for f in system galaxy empire pirate; do
    cmp -s "$W/orig.data/$f" "$W/port.data/$f" || { echo "data: $f differs"; status=1; }
done
cmp -s "$W/orig.salts" "$W/port.salts" || { echo "data: salts differ (rand sequence)"; status=1; }
(cd "$W/orig" && find . -type f | sort) > "$W/orig.files"
(cd "$W/port" && find . -type f | sort) > "$W/port.files"
diff "$W/orig.files" "$W/port.files" | sed 's/^/files: /'
while read -r f; do
    case "$f" in
    ./DATA/PLAYER.II|./Data/Player.II)
        python3 "$HERE/tools/playercmp.py" "$W/orig/$f" "$W/port/$f" || { echo "file differs: $f"; status=1; }
        continue;;
    # The salted data files are compared decrypted, above.
    ./DATA/SYSTEM.II|./DATA/GALAXY.II|./DATA/EMPIRE.II|./DATA/PIRATE.II) continue;;
    ./DATA/*.II)
        python3 "$HERE/tools/streamcmp.py" "$W/orig/$f" "$W/port/$f" > /dev/null ||
            { echo "file differs: $f"; status=1; }
        continue;;
    esac
    [ -f "$W/port/$f" ] && ! cmp -s "$W/orig/$f" "$W/port/$f" && { echo "file differs: $f"; status=1; }
done < "$W/orig.files"
exit $status
