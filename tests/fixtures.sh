#!/bin/bash
# Build the test fixtures (game directories) from the original release,
# running the original SRE.EXE under tools/dosemu.py so that every
# fixture is a state the original itself produced.
#
# usage: tests/fixtures.sh OUTDIR [sre0994b.zip]
# Needs python3 with unicorn, curl and unzip.
# SRE_STATUS=path/to/STATUS.SRE adds registered games.
set -eu
OUT=$1
ZIP=${2:-}
HERE=$(cd "$(dirname "$0")/.." && pwd)
CLOCK=${CLOCK:-1995-03-08T12:00:00}
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)

if [ -z "$ZIP" ]; then
    ZIP=$OUT/sre0994b.zip
    [ -f "$ZIP" ] || curl -fsSL -o "$ZIP" 'https://www.johndaileysoftware.com/download/?method=2&terms=1'
fi

orig() {        # orig DIR [dosemu args...]: run the original in DIR
    local d=$1; shift
    python3 -I "$HERE/tools/dosemu.py" "$d" --frozen --clock "$CLOCK" "$@" > /dev/null 2> "$d.log"
    rm -f "$d/INUSE.SR"
}

player() {      # player DIR NAME: make NAME the BBS user
    printf '%s\r\n1\r\n1\r\n25\r\n2400\r\n1\r\n60\r\n%s\r\n' "$2" "$2" > "$1/DOORFILE.SR"
}

# The release: the original SRE.EXE from the zip, and the data files
# (as UNPACK.EXE would unpack them) from game/.
rm -rf "$OUT/base"; mkdir "$OUT/base"
(cd "$OUT/base" && unzip -q "$ZIP" SRE.EXE)
cp -R "$HERE/game/." "$OUT/base/"
rm -f "$OUT/base/.gitattributes"
printf 'System.Fossil\t\tyes\r\n' > "$OUT/base/RESOURCE.DAT"
mkdir -p "$OUT/base/NEWS" "$OUT/base/BACKUP"
player "$OUT/base" "Joe Shaw"

# reset: a new game with the default settings.
rm -rf "$OUT/reset"; cp -r "$OUT/base" "$OUT/reset"
orig "$OUT/reset" RESET --local 'Joe\rMyBBS\r2Andromeda\ryy\r\r\r\r\r\rq'

# joined: Joe Shaw's empire A.
rm -rf "$OUT/joined"; cp -r "$OUT/reset" "$OUT/joined"
orig "$OUT/joined" --input ' 24yJoe\rmy nAlpha Empire\ryQQ'

# fed: joined, with plenty of food.
rm -rf "$OUT/fed"; cp -r "$OUT/joined" "$OUT/fed"
python3 "$HERE/tools/sreedit.py" "$OUT/fed" empire.0.18c=i32:5000 > /dev/null

# two: Ann Bee's empire B as well, both in year 3, with a free trade
# treaty; Ann Bee plays.
rm -rf "$OUT/two"; cp -r "$OUT/joined" "$OUT/two"
player "$OUT/two" "Ann Bee"
orig "$OUT/two" --input ' 24yAnn\rfy\x00\r\x00n\x00\rBeta Realm\ryqq'
python3 "$HERE/tools/sreedit.py" "$OUT/two" empire.0.194=i16:3 empire.1.194=i16:3 empire.0.f1=i8:2 empire.1.ed=i8:2 \
    > /dev/null

# army: two, with armies, food, money, agents, and empire B's weapons.
rm -rf "$OUT/army"; cp -r "$OUT/two" "$OUT/army"
python3 "$HERE/tools/sreedit.py" "$OUT/army" empire.0.154=i32:5000 empire.0.158=i32:3000 \
    empire.0.15c=i32:2000 empire.0.160=i32:500 empire.0.164=i32:40 empire.0.168=i32:120 \
    empire.0.2f0=i32:50 empire.0.22b=i32:200 empire.1.154=i32:6000 empire.1.158=i32:2500 \
    empire.1.15c=i32:1000 empire.1.160=i32:400 empire.1.164=i32:30 empire.1.168=i32:150 \
    empire.1.2f0=i32:40 empire.1.227=i16:7 empire.1.22b=i32:100 empire.0.18c=i32:5000 \
    empire.1.18c=i32:5000 > /dev/null

# Registered games, if a STATUS.SRE is given (SRE_STATUS=path): fedreg
# and armyreg.
if [ -n "${SRE_STATUS:-}" ]; then
    for f in fed army; do
        rm -rf "$OUT/${f}reg"; cp -r "$OUT/$f" "$OUT/${f}reg"
        cp "$SRE_STATUS" "$OUT/${f}reg/STATUS.SRE"
    done
fi

# Flavors: army, translated.
for f in TREK DUNE MEDIEVAL WILDWEST BRE; do
    rm -rf "$OUT/$f"; cp -r "$OUT/army" "$OUT/$f"
    printf 'Flavor\t\t"%s"\r\n' "$f" >> "$OUT/$f/RESOURCE.DAT"
done

ls "$OUT"
