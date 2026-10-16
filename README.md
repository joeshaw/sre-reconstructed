<p align="center"><img src="docs/banner.png" alt="Solar Realms Elite: Interstellar Imperialists. A BBS door game by Amit Patel, version 0.994b, 1995. Reconstructed by Claude Opus 5.5, 2026." width="715"></p>

# Solar Realms Elite, reconstructed

Solar Realms Elite (SRE) is a [BBS door game](https://en.wikipedia.org/wiki/BBS_door)
by Amit Patel.  In the early 1990s people dialed into bulletin board
systems by modem, and a BBS ran games like this one, called doors, for
its callers.  In SRE, the callers each build an empire in one shared
galaxy, a few turns a day.

The game's source code was lost in 1996.  This repository is a C++
reconstruction of the last release, **0.994b (March 7, 1995)**, made by
reverse engineering `SRE.EXE`.  It builds on macOS and Linux and plays
in a terminal.  It is published with Amit Patel's permission.

![The empire status screen](docs/screenshot.png)

The goal is to behave exactly like the original: the same screens,
prompts, colors, and numbers, the same random events, and game files the
original can read.  The tests check this by running the original in an
emulator alongside the reconstruction, giving both the same input, and
comparing the results (see [Testing](#testing)).  To see how it was
reverse engineered with Claude Code, see [How it was made](#how-it-was-made).

## Getting started

You need `make` and a C++ compiler: the Xcode command line tools on
macOS (`xcode-select --install`), or `build-essential` on Debian and
Ubuntu.

    git clone https://github.com/joeshaw/sre-reconstructed
    cd sre-reconstructed
    make
    tools/newgame.sh ~/sre "Your Name"
    SRE_DIR=~/sre ./sre reset      # once, as the sysop
    SRE_DIR=~/sre ./sre            # play

`make` builds `./sre`.  `tools/newgame.sh` copies the original release's
data files (help text, screens, flavors, and the research table) from
`game/` into a new game directory, and writes a door file for you.

The first time, `sre reset` asks for the sysop's name and the BBS name,
then opens the sysop menu.  The sysop is the person who runs the BBS
and the game.  Choose **2** (Reset the Game), name the galaxy, and
press Enter to accept the default for each setting.  Then press **Q**
to leave.  If `sre` exits without printing anything, the game hasn't
been set up yet, so run `sre reset` first.

The game's own help explains how to play.  The original's
documentation is in [`game/DOCS/`](game/DOCS), and Amit Patel's notes on
the game's design are on [his old site](http://www-cs-students.stanford.edu/~amitp/Articles/SRE-Design.html).

## What is reconstructed

All of the game is reconstructed: joining, turns, the economy, food,
population, the spending, selling, and planet menus, operations, covert
operations, attacks of every kind, pirates, trading fleets, diplomacy,
messages, news, the lottery, the bank, bonds, investments, events,
research, maintenance, scores, the sysop menu and its editors,
resources, translations, and flavors.

Some parts belong to the BBS and DOS rather than the game, and are left
out: the serial port and FOSSIL drivers (FOSSIL was the standard DOS
interface between doors and the modem), the sysop's status bars and hot
keys on the local screen, sysop chat, EMS and XMS memory swapping, and
the VGA palette.  The terminal stands in for the caller's modem.  In a
local session (`sre sysop`, `sre reset`, or local play), it also stands
in for the sysop's screen and keyboard.

The game is always registered, so it has everything registration added
to the original: the bank, bonds, investments, the Super Daily lottery,
nuclear, chemical, and psionic weapons, and the full reset options.  The
registered name and the bank's name come from `STATUS.SRE` if there is
one, without checking its key.  Otherwise the game is registered to the
sysop.

The original only accepts dates up to December 11, 1999, so the
reconstruction moves the clock back 28 years (see `SRE_DATE_SHIFT`
below).

## Playing

`sre sysop` opens the sysop menu again after the first reset: the empire
viewer, the player and pirate editors, and messages.  Commands are not
case sensitive.

Other players can share the game directory.  As on a BBS, only one
person can play at a time.  Before each one plays, change the first
line of `DOORFILE.SR` to their name.

### The door file

The BBS used `DOORFILE.SR` to tell SRE about the caller.  It has eight
lines, and the game reads the first seven:

    Your Name       the user's name on the BBS
    1               ANSI color (1/0)
    1               IBM (PC) characters (1/0)
    25              lines on the screen
    38400           baud rate
    1               COM port; 0 for a local (sysop) session
    60              minutes left
    Your Name       the user's real name

`tools/newgame.sh` fills in the terminal's height, and the name defaults
to `$USER`.

### Settings

Settings in `RESOURCE.DAT` work as described in the original's
[`DOCS/RESOURCE.DOC`](game/DOCS/RESOURCE.DOC), except those for the
parts left out (serial ports, FOSSIL, display, status bars, and
swapping), which are ignored.  On the command line, on/off settings can
be given as `/Name`, `/NoName`, or `/Name=Y` (or `-` instead of `/`).
The original's documents use the IBM PC character set, so their box
drawings look garbled on GitHub.

Environment variables:

- `SRE_DIR`: the game directory (default: the current directory).
- `SRE_CHARSET`: `utf8` or `cp437`.  The game's text uses the IBM PC
  character set.  When this is unset, the game converts it to UTF-8 if
  its output is a terminal and the locale is UTF-8 (the default on
  macOS).
- `SRE_DATE_SHIFT`: years to move the clock back (default 28).  The
  original only accepts dates from December 25, 1979 to December 11,
  1999.  A shift of 28 years keeps the weekdays and leap years the same,
  and works until December 11, 2027.  After that, set it to another
  multiple of 4 that brings the date back into that range, such as 32
  (which works through 2031), and don't change it during a game.  Past
  the limit, `sre` exits without printing anything.

## Running it on the web

`server/` contains `sre-server`, which runs the game the way a BBS ran a
door: one shared game and one player at a time, each playing in a
terminal in their browser on a page styled like a BBS.  It needs Go (see
`server/go.mod` for the version):

    make sre server
    tools/newgame.sh ~/sre-web/game
    server/sre-server -listen :8000 -game ~/sre-web/game -sre ./sre \
        -keys ~/sre-web/keys.json -sysops YourName

Browsers only allow the page's login keys over HTTPS (or on
`localhost`), so serve it behind a proxy that handles TLS, such as Caddy
or nginx.  [`server/README.md`](server/README.md) has more details,
including how to run it on exe.dev.

## Testing

`tests/smoke.sh` needs the build and Python 3.7 or later, with no extra
packages.  It sets up a game from `game/`, resets it, joins two players,
plays turns, and runs the next day's maintenance.  Then it compares the
output and the game's data with the recordings in `tests/smoke/`, which
match the original's byte for byte.  CI runs this test on macOS and
Linux.

    make && tests/smoke.sh

The full tests compare against the original.  `tools/dosemu.py` (see
[How it was made](#how-it-was-made)) runs the original `SRE.EXE`, and
the reconstruction runs on a copy of the same game with the same input.
The tests compare the output byte for byte, the game's files after
decrypting them (leaving out the bytes the original never initializes),
and the random number sequence.  They need `curl`, `unzip`, and Python
with [Unicorn](https://www.unicorn-engine.org/):

    python3 -m venv .venv && . .venv/bin/activate
    pip install unicorn pyte capstone   # pyte for tools/render.py, capstone for tools/disasm.py
    tests/fixtures.sh /tmp/srefix       # games made by the original, some with fields edited
    tests/run.sh /tmp/srefix            # the scenarios in tests/scenarios.txt
    tests/fuzz.sh /tmp/srefix army 1 20 # random play, seeds 1 to 20
    tests/fuzzchain.sh /tmp/srefix two 1 # two players taking turns over several days

`tests/fixtures.sh` downloads the original release, `sre0994b.zip`, from
[John Dailey Software](https://www.johndaileysoftware.com/), verifies
its SHA-256 checksum, and runs the `SRE.EXE` inside it to create the
games that the scenarios start from.  You can also give it the zip:
`tests/fixtures.sh DIR path/to/sre0994b.zip`.

The tests run with `SRE_UNREGISTERED` set, so registration works as in
the original: the game is unregistered unless there is a valid
`STATUS.SRE`.  If you have one from a registered copy of the original,
set `SRE_STATUS=path/to/STATUS.SRE` and `tests/fixtures.sh` also makes
registered games for the bank, bond, and lottery scenarios.  Without it,
`tests/run.sh` skips those scenarios.  `SRE_CLOCK` gives the tests a
frozen clock, in seconds since 1970.  It is read as local time and is
not shifted, so it must be a date the original accepts.

## How it was made

Claude reconstructed the game in Claude Code, directed by Joe Shaw.  It
started in an empty repository with this prompt:

> solar realms elite was a bbs door from the 90s. the source code was lost
> in 1996.
>
> i want you to reverse engineer the last release (available at
> https://www.johndaileysoftware.com/download/?method=2&terms=1) from the
> binary and produce source code we can compile on a modern mac. it is not
> necessary to reconstruct the BBS/fossil parts, but the gameplay and text
> user interface need to be reconstructed perfectly from the original.
>
> Additional info from the original author can be found on
> http://www-cs-students.stanford.edu/~amitp/Articles/SRE-Design.html and
> other pages on his site.
>
> Use this repository to build it

(That download link no longer works.  `tests/fixtures.sh` uses one that
does.)

The work went in five steps.

1. Taking the binary apart.  `SRE.EXE` is a Borland C++ large-model
   program, and most of its code is in VROOMM overlays, Borland's system
   for loading parts of a program from the `.EXE` only when they are
   needed.  `tools/deoverlay.py` turns it into a plain executable with
   every segment in place, so it can be disassembled as a whole.  Ghidra
   produced the function list and call graph in `tools/re/`.
   `tools/disasm.py`, built on [Capstone](https://www.capstone-engine.org/),
   produces listings annotated with names and strings, and
   `tools/vtables.py` finds the virtual tables.  `tools/lift.py`, a
   pseudo-C lifter for Borland's code, makes the listings easier to read.
2. Running the original.  `tools/dosemu.py` emulates just enough of DOS,
   the BIOS, and a FOSSIL driver, on top of
   [Unicorn](https://www.unicorn-engine.org/), to run `SRE.EXE` with
   scripted input on its serial line or keyboard and a frozen clock.  It
   captures the game's output, screen, and files.  With it, any question
   about how the original behaves can be answered by running it.
3. Writing the source.  Each part of the original was rewritten in turn
   in C++, keeping the original's structure: one file per part of the
   program (usually one code segment), the original's classes (streams,
   menus, and the objects stored in the data files), and each function's
   address in a comment, in the 16-bit `segment:offset` form, so it can
   be found in the original binary.  Where the game depends on how the
   Borland runtime and compiler behave, that behavior is reproduced
   exactly: the random number generator, the date and time conversions,
   the `ctype` table, text-mode file reads, and 16-bit integer
   arithmetic.
4. Checking it.  Each part was checked by running it and the original on
   copies of the same game with the same input, and comparing the
   output, the game's files, and the random number sequence (observed
   through the encryption salts it produces).  The scenarios in
   `tests/scenarios.txt` go through the menus and features, and random
   play (`tests/fuzz.sh`, and `tests/fuzzchain.sh` for several players
   over multiple days) found what the scenarios missed.  The registered
   features were checked with a registration file, which is not included
   here.
5. Naming.  At first, names came from the disassembly, such as `d188`
   for the field at that offset in a record.  Once everything worked,
   each name was replaced with one based on how the game uses it
   (`pollution`), and the tests confirmed that nothing changed.

## Repository layout

- `src/`: the reconstructed game, with the original's addresses in the
  comments.
- `game/`: the original release's data files and documentation.
- `server/`: `sre-server`, the web front end.
- `tests/`: the smoke test and its recordings, the scenarios, and the
  scripts that compare with the original.
- `tools/`: helpers for comparing runs (`difftest.sh`, `localtest.sh`,
  `render.py`, `localdiff.py`, `memdiff.py`, `playercmp.py`, and
  `streamcmp.py`), for setting up and editing games (`newgame.sh`,
  `sredata.py`, and `sreedit.py`), and for reverse engineering
  (`deoverlay.py`, `disasm.py`, `lift.py`, `vtables.py`, and
  `dosemu.py`).
- `tools/re/`: Ghidra's list of the original's functions (address, size,
  callers, and callees).  `names.txt` gives names to about 120 of them,
  the ones identified first (the Borland runtime, streams, files,
  resources, and strings), and `tools/disasm.py` and `tools/lift.py`
  show calls to them by name.  `globals.txt` is for names of data
  addresses, and is empty.

## The Rust port

The `rust` branch has the same game ported to Rust, module by module
from the C++.  Its output and files are byte for byte the same as the
C++ build's, and so the original's, in every scenario and in thousands
of random games.  It needs only Rust and Cargo (`git checkout rust`,
then `make`), and has no dependencies beyond `libc`.

## Credits

Solar Realms Elite was written by Amit Patel, now of
[Red Blob Games](https://www.redblobgames.com/).  The game is
distributed by [John Dailey Software](https://www.johndaileysoftware.com/).

The reconstruction relies on [Ghidra](https://ghidra-sre.org/),
[Capstone](https://www.capstone-engine.org/), and
[Unicorn](https://www.unicorn-engine.org/), and the web front end
includes [xterm.js](https://xtermjs.org/) (see
`server/web/vendor/XTERM-LICENSE`).
