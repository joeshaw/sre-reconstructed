#!/bin/bash
# Differential test of the local (sysop) functions: run the original with
# local keyboard input and the reconstruction with the same keys on its
# terminal, then compare the local screen text and the data.
#
# usage: tools/localtest.sh TEMPLATE_DIR WORK_DIR 'keys' [args...]
set -u
T=$1; W=$2; IN=$3; shift 3
HERE=$(cd "$(dirname "$0")/.." && pwd)
CLOCK=${CLOCK:-1995-03-08T12:00:00}
SECS=$(python3 -c "import datetime,calendar;print(calendar.timegm(datetime.datetime.fromisoformat('$CLOCK').timetuple()))")
rm -rf "$W"; mkdir -p "$W"
cp -r "$T" "$W/orig"; cp -r "$T" "$W/port"
timeout 900 python3 -I "$HERE/tools/dosemu.py" "$W/orig" "$@" --local "$IN" --frozen --clock "$CLOCK" \
    --local-out "$W/orig.out" --screen 2> "$W/orig.err"
printf '%b' "$IN" | (cd "$W/port" && SRE_CLOCK=$SECS timeout 60 "$HERE/sre" "$@" > "$W/port.out")
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
    case "$f" in ./DATA/*.II|./Data/*.II) continue;; esac
    [ -f "$W/port/$f" ] && ! cmp -s "$W/orig/$f" "$W/port/$f" && { echo "file differs: $f"; status=1; }
done < "$W/orig.files"
exit $status
