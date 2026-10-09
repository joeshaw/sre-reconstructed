#!/bin/bash
# Random play: feed the original and the reconstruction the same random
# keys and compare their output.  (The games' files are compared only
# when the original got to exit.)
#
# usage: tests/fuzz.sh FIXDIR FIXTURE FIRST-SEED LAST-SEED [KEYS]
set -u
FIX=$(cd "$1" && pwd); T=$2; A=$3; B=$4; N=${5:-300}
HERE=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$FIX/fuzz"
fail=0
for seed in $(seq "$A" "$B"); do
    input=$(python3 - "$seed" "$N" <<'PY'
import random, sys
r = random.Random(int(sys.argv[1]))
keys = ['\\r'] * 30 + list('0123456789') * 2 + list('yYnNqQcbsatm*') + \
       ['A', 'B', '\\r', ' ', '100\\r', '5\\r', '1000\\r', '/s\\r', 'Hi\\r']
print(' 2' + ''.join('\\x00' + r.choice(keys) for _ in range(int(sys.argv[2]))), end='')
PY
)
    w=$FIX/fuzz/$T-$seed
    out=$("$HERE/tools/difftest.sh" "$FIX/$T" "$w" "$input" 2>&1)
    if echo "$out" | grep -q '^output: same'; then
        if echo "$out" | grep -q 'result=exit' && echo "$out" | grep -q '^data:\|^file differs'; then
            echo "FAIL $T $seed (data)"; echo "$out" | sed 's/^/    /' | head -8; fail=1
        else
            echo "PASS $T $seed"
        fi
    else
        echo "FAIL $T $seed"; echo "$out" | sed 's/^/    /' | head -8; fail=1
    fi
done
exit $fail
