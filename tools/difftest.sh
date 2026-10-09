#!/bin/bash
# Differential test: run the original SRE.EXE (under tools/dosemu.py) and
# the reconstruction on copies of the same game directory with the same
# remote input and a frozen clock, then compare everything.
#
# usage: tools/difftest.sh TEMPLATE_DIR WORK_DIR 'input' [args...]
set -u
T=$1; W=$2; IN=$3; shift 3
HERE=$(cd "$(dirname "$0")/.." && pwd)
CLOCK=${CLOCK:-1995-03-08T12:00:00}
SECS=$(python3 -c "import datetime,calendar;print(calendar.timegm(datetime.datetime.fromisoformat('$CLOCK').timetuple()))")
rm -rf "$W"; mkdir -p "$W"
cp -r "$T" "$W/orig"; cp -r "$T" "$W/port"
timeout 900 python3 -I "$HERE/tools/dosemu.py" "$W/orig" "$@" --input "$IN" --frozen --clock "$CLOCK" --out "$W/orig.out" 2> "$W/orig.err"
printf '%b' "$IN" | (cd "$W/port" && SRE_CLOCK=$SECS timeout 60 "$HERE/sre" "$@" > "$W/port.out")
echo "port exit=$?"
tail -1 "$W/orig.err"
status=0
if cmp -s "$W/orig.out" "$W/port.out"; then echo "output: same ($(wc -c < "$W/orig.out") bytes)"; else
    status=1
    python3 - "$W/orig.out" "$W/port.out" <<'PY'
import sys
a=open(sys.argv[1],'rb').read(); b=open(sys.argv[2],'rb').read()
i=next((i for i in range(min(len(a),len(b))) if a[i]!=b[i]),min(len(a),len(b)))
print('output differs at byte %d (orig %d, port %d bytes)'%(i,len(a),len(b)))
print(' orig:',a[max(0,i-60):i+80]); print(' port:',b[max(0,i-60):i+80])
PY
fi
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
    case "$f" in ./DATA/*.II|./Data/*.II) continue;; esac
    [ -f "$W/port/$f" ] && ! cmp -s "$W/orig/$f" "$W/port/$f" && { echo "file differs: $f"; status=1; }
done < "$W/orig.files"
exit $status
