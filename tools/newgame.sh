#!/bin/bash
# Set up a directory to play Solar Realms Elite in: the original release's
# data files (help text, screens, flavors, research table, docs) from
# game/, and a DOORFILE.SR for the player.
#
# usage: tools/newgame.sh DIR ["Your Name"]
set -eu
HERE=$(cd "$(dirname "$0")/.." && pwd)
DIR=$1
NAME=${2:-${USER:-Player}}

mkdir -p "$DIR"
cp -R "$HERE/game/." "$DIR/"
rm -f "$DIR/.gitattributes"
cd "$DIR"
mkdir -p DATA NEWS BACKUP

# The door file: name, ANSI, IBM characters, page length, baud, port,
# minutes, real name.  Port 1 makes the terminal the player's line.
ROWS=$(tput lines 2> /dev/null || echo 25)
printf '%s\r\n1\r\n1\r\n%s\r\n38400\r\n1\r\n60\r\n%s\r\n' "$NAME" "$ROWS" "$NAME" > DOORFILE.SR

SRE=$HERE/sre
cat <<MSG
Game set up in $DIR for $NAME.
First reset the game (as the sysop):   SRE_DIR=$DIR $SRE reset
Then play:                             SRE_DIR=$DIR $SRE
MSG
