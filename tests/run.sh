#!/bin/bash
# Run the differential test scenarios (tests/scenarios.txt) against the
# fixtures made by tests/fixtures.sh: each runs the original under the
# emulator and the reconstruction on copies of a game, with the same input,
# and compares the output and the game's files.
#
# usage: tests/run.sh FIXDIR [NAME-PATTERN]
set -u
FIX=$(cd "$1" && pwd)
PAT=${2:-}
HERE=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$FIX/work"
pass=0; fail=0; failed=
while IFS='|' read -r name mode template clock player args input; do
    case "$name" in ''|'#'*) continue;; esac
    [ -n "$PAT" ] && ! [[ "$name" == $PAT ]] && continue
    if [[ "$template" == @* ]]; then
        src=$FIX/work/${template#@}/orig
    else
        src=$FIX/$template
    fi
    w=$FIX/work/$name
    tmpl=$FIX/work/$name.template
    rm -rf "$tmpl"; cp -r "$src" "$tmpl"; rm -f "$tmpl/INUSE.SR"
    if [ -n "$player" ]; then
        printf '%s\r\n1\r\n1\r\n25\r\n2400\r\n1\r\n60\r\n%s\r\n' "$player" "$player" > "$tmpl/DOORFILE.SR"
    fi
    if [ "$mode" = local ]; then
        script=$HERE/tools/localtest.sh
    else
        script=$HERE/tools/difftest.sh
    fi
    # {N*KEYS} repeats KEYS N times.
    input=$(python3 -c 'import re,sys; print(re.sub(r"\{(\d+)\*([^}]*)\}", lambda m: m.group(2) * int(m.group(1)), sys.argv[1]), end="")' "$input")
    # shellcheck disable=SC2086
    out=$(CLOCK=${clock:-1995-03-08T12:00:00} "$script" "$tmpl" "$w" "$input" $args 2>&1)
    if [ $? -eq 0 ]; then
        pass=$((pass + 1)); echo "PASS $name"
    else
        fail=$((fail + 1)); failed="$failed $name"; echo "FAIL $name"
        echo "$out" | sed 's/^/    /' | head -12
    fi
done < "$HERE/tests/scenarios.txt"
echo "$pass passed, $fail failed${failed:+:$failed}"
[ $fail -eq 0 ]
