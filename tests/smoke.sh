#!/bin/bash
# Smoke test that needs nothing but the build and game/: set up a game,
# reset it as the sysop, join two players, play turns and run the next
# day's maintenance, with fixed input and a frozen clock, and compare the
# output and the game's data with tests/smoke/ (recorded from runs that
# are byte for byte the same as the original's).
#
# usage: tests/smoke.sh [--record]
set -u
HERE=$(cd "$(dirname "$0")/.." && pwd)
SRE=${SRE:-$HERE/sre}
EXP=$HERE/tests/smoke
RECORD=0
[ "${1:-}" = --record ] && RECORD=1
W=$(mktemp -d)
trap 'rm -rf "$W"' EXIT
G=$W/game

# A new game from the release's data files, as tests/fixtures.sh makes it.
cp -R "$HERE/game/." "$G/"
rm -f "$G/.gitattributes"
printf 'System.Fossil\t\tyes\r\n' > "$G/RESOURCE.DAT"
mkdir -p "$G/DATA" "$G/NEWS" "$G/BACKUP"

fail=0
# step NAME CLOCK PLAYER ARGS INPUT (as in tests/scenarios.txt)
step() {
    local name=$1 clock=$2 player=$3 args=$4 input=$5
    printf '%s\r\n1\r\n1\r\n25\r\n2400\r\n1\r\n60\r\n%s\r\n' "$player" "$player" > "$G/DOORFILE.SR"
    rm -f "$G/INUSE.SR"
    local secs in
    secs=$(python3 -c "import datetime,calendar;print(calendar.timegm(datetime.datetime.fromisoformat('$clock').timetuple()))")
    in=$(python3 -c 'import re,sys; print(re.sub(r"\{(\d+)\*([^}]*)\}", lambda m: m.group(2) * int(m.group(1)), sys.argv[1]), end="")' "$input")
    printf '%b' "$in" > "$W/$name.in"
    (cd "$G" && SRE_TESTMARK=1 SRE_UNREGISTERED=1 SRE_CLOCK=$secs "$SRE" $args < "$W/$name.in" > "$W/$name.out")
    # The data files, decrypted (the other files hold uninitialized memory).
    python3 "$HERE/tools/sredata.py" "$G" --dump "$W/$name.dump" > /dev/null
    (cd "$W/$name.dump" && for f in system galaxy empire pirate; do
        printf '%s  %s\n' "$(python3 -c 'import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$f")" "$f"
    done) > "$W/$name.data"
    if [ $RECORD = 1 ]; then
        cp "$W/$name.out" "$EXP/$name.out"
        cp "$W/$name.data" "$EXP/$name.data"
        echo "recorded $name"
    elif cmp -s "$W/$name.out" "$EXP/$name.out" && cmp -s "$W/$name.data" "$EXP/$name.data"; then
        echo "PASS $name"
    else
        echo "FAIL $name"
        cmp "$W/$name.out" "$EXP/$name.out"
        diff "$W/$name.data" "$EXP/$name.data"
        fail=1
    fi
}

mkdir -p "$EXP"
T=1995-03-08T12:00:00
step reset "$T" "Joe Shaw" RESET 'Joe\rMyBBS\r2Andromeda\ryy\r\r\r\r\r\rq'
step join "$T" "Joe Shaw" "" ' 24yJoe\rmy nAlpha Empire\ryQQ'
step turns "$T" "Joe Shaw" "" ' 24Q  y{38*\x00\r}\x00n{30*\x00\r}'
step join2 "$T" "Ann Bee" "" ' 24yAnn\rfy\x00\r\x00n\x00\rBeta Realm\ryqq'
step maint 1995-03-09T12:00:00 "Ann Bee" "" ' 2\x001\x003\x00\r\x00q\x00q\x00\r'
exit $fail
