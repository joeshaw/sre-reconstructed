# sre-server

Solar Realms Elite on the web, run the way a BBS ran a door game: one
shared game on the server, one player at a time, each in their own
session of the game on a terminal in their browser.

- **Players are keys.**  The page makes an ECDSA P-256 key the first
  time a browser visits, and keeps it in IndexedDB, unreadable even by
  the page.  To connect, the page signs a challenge from the server.  A
  new key's holder picks a name, first come, first served.  The server
  keeps the keys and names in `keys.json`.  A player who clears their
  browser's data, or moves to another browser, is a new player.
- **One line.**  Someone arriving while another plays is told who is
  playing and waits; the next in line gets in when the line is free.
- **Hanging up is safe.**  Closing the tab hangs up the session, as a
  dropped modem connection did.  The game saves what was played and
  exits.
- **The sysop:** names given with `-sysops` get the "Sysop menu" button.
  It runs `sre reset`, which sets the game up the first time and opens
  the sysop menu after that.

## Building

    make sre server     # ./sre, the game, and server/sre-server, with the page built in

## Running

    tools/newgame.sh /srv/sre/game        # the game's files
    server/sre-server -listen :8000 -game /srv/sre/game -sre ./sre \
        -keys /srv/sre/keys.json -sysops YourName

Open the page, press **Sysop menu**, and pick your name.  It must be one
given with `-sysops`.  Enter the sysop's and BBS's names, then choose
**2** to reset the game.  After that, players press **Play**.

## On exe.dev

The game is small.  A session is one process of a few MB, using almost
no CPU, and only one runs at a time.  The game's files are under a
megabyte.  So the smallest VM will do.  It needs enough memory to build
the game and the server on it (2 GB is comfortable), and the default
disk has room to spare:

    ssh exe.dev new --name=sre --cpu=1 --memory=2GB --disk=10GB

An exe.dev VM has no public IP of its own.  The public reaches it
through exe.dev's HTTPS proxy, which handles TLS.  So the server listens
on plain HTTP and the page connects back with `wss://`.

    ssh exe.dev share port <vm> 8000
    ssh exe.dev share set-public <vm>

Players then visit `https://<vm>.exe.xyz/`.  Run the server under
systemd (or similar) so it restarts with the VM.  Back up the game
directory and `keys.json`.

exe.dev's own SSH gateway answers `ssh <vm>.exe.xyz`, for people with
access to the VM.  So a door for players' own ssh clients would need
another host.  The page is the way in here.
