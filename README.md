# Solar Realms Elite, reconstructed

Solar Realms Elite (SRE) is a BBS door game by Amit Patel.  Its source
code was lost in 1996.  This repository is a reconstruction of the last
release, **0.994b (7 March 1995)**, in C++, made by reverse engineering
`SRE.EXE`.  It builds on a modern Mac (or Linux) and plays in a terminal.

The aim is the original's exact behaviour: the same screens, prompts,
colours and numbers, the same random events, and game files the original
can read.  That is checked by running the original under an emulator next
to the reconstruction with the same input and comparing everything (see
[Testing](#testing)).

## Building

On macOS, with the Xcode command line tools (`xcode-select --install`):

    make

This builds `./sre`.  Any C++11 compiler works (`make CXX=g++` on
Linux).

## Playing

The game needs the data files of the original release (help text,
screens, flavors, the research table).  `tools/newgame.sh` downloads the
release from [John Dailey Software](https://www.johndaileysoftware.com/),
unpacks it into a directory and writes a door file for you:

    brew install p7zip                  # to unpack the release's ARJ archive
    tools/newgame.sh ~/sre "Your Name"
    cd ~/sre
    /path/to/sre Reset                  # once, as the sysop: set up the galaxy
    /path/to/sre                        # play

`sre Reset` asks for the sysop's and the BBS's names and opens the sysop
menu; choose **2** (Reset the Game).  Afterwards `sre Sysop` opens the
sysop menu again (empire viewer, player and pirate editors, messages).

Other players can share the directory: give each their own copy of
`DOORFILE.SR` (or edit its first and last lines, the user's name) and run
`sre` from the directory.  As on a BBS, one player at a time.

### The door file

`DOORFILE.SR` is what the BBS used to tell SRE about the caller, eight
lines:

    Your Name       the user's name on the BBS
    1               ANSI colour (1/0)
    1               IBM (PC) characters (1/0)
    24              lines on the screen
    38400           baud rate
    1               COM port; 0 for a local (sysop) session
    60              minutes left
    Your Name       the user's real name

Settings from the original's `RESOURCE.DAT` and command line options
(`-Name=value`) work as described in the original's `DOCS/RESOURCE.DOC`.

### Environment

- `SRE_DIR`: the game directory (default: the current directory).
- `SRE_CHARSET`: `utf8` or `cp437`.  The game's text uses the IBM PC
  character set; on a UTF-8 terminal (the default on macOS) it is
  translated, unless this says `cp437`.
- `SRE_DATE_SHIFT`: years to move the clock back (default 28).  The
  original accepts dates from 1980 to November 1999 only; 28 years keeps
  the calendar the same and works until the end of 2027.  Set a larger
  multiple of 4 after that, and don't change it during a game.
- `SRE_CLOCK`: a fixed time (seconds since 1970) instead of the clock,
  for testing.

## What is reconstructed

All of the game: joining, turns, the economy, food, population, the
spending, selling and planet menus, operations, covert operations,
attacks of every kind, pirates, trading fleets, diplomacy, messages,
news, the lottery, the bank, bonds and investments (registered games),
events, research, maintenance, scores, the sysop menu and its editors,
resources, translations and flavors.  The source follows the original's
structure: one file per part of the program, with the original's
addresses (`segment:offset`) in the comments.

Left out, as parts of the BBS and DOS environment rather than the game:
the FOSSIL and serial port drivers, the sysop's status bars and hot keys
on the local screen, sysop chat, EMS/XMS swapping and the VGA palette.
The terminal stands in for both the caller's modem and, when nobody is
connected remotely (`sre Sysop`, `sre Reset`, local play), the sysop's
screen and keyboard.

The original's registration is reproduced: an unregistered game behaves
as the original's unregistered version.

## Testing

`tools/dosemu.py` is a small DOS emulator (on
[Unicorn](https://www.unicorn-engine.org/)) that runs the original
`SRE.EXE` with scripted input on its serial line or keyboard and a
frozen clock.  The tests run the original and the reconstruction on
copies of the same game and compare the output byte for byte, the game
files, and the random number sequence.

    pip install unicorn pyte            # pyte only for tools/render.py
    tests/fixtures.sh /tmp/srefix       # games made by the original itself
    tests/run.sh /tmp/srefix            # the scenarios in tests/scenarios.txt
    tests/fuzz.sh /tmp/srefix army 1 20 # random play, seeds 1 to 20
    tests/fuzzchain.sh /tmp/srefix two 1 # two players taking turns over days

With `SRE_STATUS=path/to/STATUS.SRE`, `tests/fixtures.sh` also makes
registered games, for the scenarios of the bank, bonds and lottery.

`tools/difftest.sh` and `tools/localtest.sh` compare one run; the other
tools are the reverse engineering kit (de-overlaying the executable,
disassembly, a pseudo-C lifter, data file decoding and editing).

## Credits

Solar Realms Elite was written by Amit Patel.  His notes on its design
are at <http://www-cs-students.stanford.edu/~amitp/Articles/SRE-Design.html>.
The game is distributed by John Dailey Software.
