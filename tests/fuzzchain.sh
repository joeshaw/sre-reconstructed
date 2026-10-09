#!/bin/bash
# Random play over several sessions: Ann Bee and Joe Shaw take turns on
# the same game, a day apart every other session, each session's game
# being the one the original left.  Compares each session's output (and
# the files, when the original got to exit).
#
# usage: tests/fuzzchain.sh FIXDIR FIXTURE SEED [SESSIONS] [KEYS]
set -u
FIX=$(cd "$1" && pwd); T=$2; SEED=$3; S=${4:-6}; N=${5:-250}
HERE=$(cd "$(dirname "$0")/.." && pwd)
base=$FIX/chain/$T-$SEED
rm -rf "$base"; mkdir -p "$base"
cp -r "$FIX/$T" "$base/game0"
for i in $(seq 1 "$S"); do
    if [ $((i % 2)) = 1 ]; then who="Ann Bee"; else who="Joe Shaw"; fi
    day=$(( (i - 1) / 2 + 8 ))
    clock=$(printf '1995-03-%02dT%02d:00:00' "$day" $((10 + i)))
    g=$base/game$((i - 1))
    rm -f "$g/INUSE.SR"
    printf '%s\r\n1\r\n1\r\n25\r\n2400\r\n1\r\n60\r\n%s\r\n' "$who" "$who" > "$g/DOORFILE.SR"
    input=$(python3 - "$SEED$i" "$N" <<'PY'
import random, sys
r = random.Random(int(sys.argv[1]))
keys = ['\\r'] * 30 + list('0123456789') * 2 + list('yYnNqQcbsatmdvwi*') + \
       ['A', 'B', '\\r', ' ', '100\\r', '5\\r', '1000\\r', '25000\\r', '/s\\r', 'Hi\\r']
print(' 2' + ''.join('\\x00' + r.choice(keys) for _ in range(int(sys.argv[2]))), end='')
PY
)
    out=$(CLOCK=$clock "$HERE/tools/difftest.sh" "$g" "$base/run$i" "$input" 2>&1)
    if ! echo "$out" | grep -q '^output: same' ||
       { echo "$out" | grep -q 'result=exit' && echo "$out" | grep -q '^data:\|^file differs'; }; then
        echo "FAIL $T $SEED session $i ($who, $clock)"
        echo "$out" | sed 's/^/    /' | head -8
        exit 1
    fi
    cp -r "$base/run$i/orig" "$base/game$i"
done
echo "PASS $T $SEED ($S sessions)"
