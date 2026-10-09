#!/bin/bash
# Set up a directory to play Solar Realms Elite in: the data files of the
# original release (help text, screens, flavors, research table) and a
# DOORFILE.SR for the player.
#
# usage: tools/newgame.sh DIR ["Your Name"] [sre0994b.zip]
# Needs curl, unzip, and 7z (macOS: brew install p7zip) or unar.
set -eu
DIR=$1
NAME=${2:-${USER:-Player}}
ZIP=${3:-}
URL='https://www.johndaileysoftware.com/download/?method=2&terms=1'

mkdir -p "$DIR"
cd "$DIR"
if [ -z "$ZIP" ]; then
    ZIP=sre0994b.zip
    [ -f "$ZIP" ] || curl -fsSL -o "$ZIP" "$URL"
fi
unzip -q -o "$ZIP" -x INSTALL.EXE
if command -v 7z > /dev/null; then
    7z x -y UNPACK.EXE > /dev/null
elif command -v unar > /dev/null; then
    unar -q -f UNPACK.EXE > /dev/null
else
    echo "newgame: need 7z or unar to unpack UNPACK.EXE (an ARJ archive)" >&2
    exit 1
fi
rm -f UNPACK.EXE
mkdir -p DATA NEWS BACKUP

# The door file: name, ANSI, IBM characters, page length, baud, port,
# minutes, real name.  Port 1 makes the terminal the player's line.
ROWS=$(tput lines 2> /dev/null || echo 25)
printf '%s\r\n1\r\n1\r\n%s\r\n38400\r\n1\r\n60\r\n%s\r\n' "$NAME" "$ROWS" "$NAME" > DOORFILE.SR

cat <<EOF
Game set up in $DIR for $NAME.
First reset the game (as the sysop):   cd $DIR && sre Reset
Then play:                             cd $DIR && sre
EOF
