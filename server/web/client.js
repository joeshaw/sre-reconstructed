// The page's side of sre-server: a key that identifies the player, kept
// in this browser, and a terminal connected to a session of the game.
'use strict';

// The PC's sixteen colours, the darkest two lifted to be readable on black.
const VGA = {
  black: '#000000', red: '#aa0000', green: '#00aa00', yellow: '#aa5500',
  blue: '#3a4fd8', magenta: '#aa00aa', cyan: '#00aaaa', white: '#aaaaaa',
  brightBlack: '#6e6e6e', brightRed: '#ff5555', brightGreen: '#55ff55',
  brightYellow: '#ffff55', brightBlue: '#5f7bff', brightMagenta: '#ff55ff',
  brightCyan: '#55ffff', brightWhite: '#ffffff',
  background: '#000000', foreground: '#aaaaaa', cursor: '#aaaaaa',
};

// 80 columns at a readable size: as large as fits, but never under 12px
// (narrow screens scroll the terminal sideways instead).
const fontSize = () => Math.max(12, Math.min(18, Math.floor((window.innerWidth - 64) / 48)));
const term = new Terminal({
  cols: 80, rows: 25, theme: VGA, fontSize: fontSize(),
  fontFamily: getComputedStyle(document.body).fontFamily,
  drawBoldTextInBrightColors: true, cursorBlink: true,
});
term.open(document.getElementById('term'));
window.addEventListener('resize', () => { term.options.fontSize = fontSize(); });
const status = (s) => { document.getElementById('status').textContent = s; };
const buttons = document.querySelectorAll('.menu button');
const setConnected = (on) => buttons.forEach((b) => { b.disabled = on; });

// The player's key: ECDSA P-256, made once and kept in IndexedDB.  It is
// not extractable, so this page can sign with it but nothing can read it.
function db() {
  return new Promise((ok, fail) => {
    const r = indexedDB.open('sre', 1);
    r.onupgradeneeded = () => r.result.createObjectStore('keys');
    r.onsuccess = () => ok(r.result);
    r.onerror = () => fail(r.error);
  });
}

async function playerKey() {
  const d = await db();
  const get = () => new Promise((ok, fail) => {
    const r = d.transaction('keys').objectStore('keys').get('player');
    r.onsuccess = () => ok(r.result);
    r.onerror = () => fail(r.error);
  });
  let pair = await get();
  if (!pair) {
    pair = await crypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-256' }, false,
                                           ['sign', 'verify']);
    await new Promise((ok, fail) => {
      const t = d.transaction('keys', 'readwrite');
      t.objectStore('keys').put(pair, 'player');
      t.oncomplete = ok;
      t.onerror = () => fail(t.error);
    });
  }
  return pair;
}

const b64 = (buf) => btoa(String.fromCharCode(...new Uint8Array(buf)));
const unb64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));

let ws = null;

term.onData((s) => {
  if (ws && ws.readyState === WebSocket.OPEN) ws.send(new TextEncoder().encode(s));
});

async function connect(mode) {
  if (ws) return;
  const pair = await playerKey();
  const spki = await crypto.subtle.exportKey('spki', pair.publicKey);
  term.reset();
  term.focus();
  setConnected(true);
  status('Connecting...');
  const url = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/session';
  ws = new WebSocket(url);
  ws.binaryType = 'arraybuffer';
  ws.onmessage = async (ev) => {
    if (typeof ev.data !== 'string') {
      term.write(new Uint8Array(ev.data));
      return;
    }
    const msg = JSON.parse(ev.data);
    if (msg.type === 'challenge') {
      const sig = await crypto.subtle.sign({ name: 'ECDSA', hash: 'SHA-256' }, pair.privateKey,
                                           unb64(msg.nonce));
      ws.send(JSON.stringify({ type: 'auth', key: b64(spki), sig: b64(sig), mode,
                               rows: term.rows }));
      status('Connected');
    } else if (msg.type === 'hello') {
      status('Connected as ' + msg.name);
    }
  };
  ws.onclose = () => {
    ws = null;
    setConnected(false);
    status('Disconnected.  Press P to play again');
    term.write('\r\n\x1b[0;1;34m[\x1b[37mNO CARRIER\x1b[34m]\x1b[0m\r\n');
  };
}

document.getElementById('play').onclick = () => connect('play');
document.getElementById('sysop').onclick = () => connect('sysop');
// The menu's keys, while not connected (once connected, keys go to the game).
document.addEventListener('keydown', (e) => {
  if (ws || e.ctrlKey || e.metaKey || e.altKey) return;
  const b = document.querySelector(`.menu button[data-key="${e.key.toLowerCase()}"]`);
  if (b) { e.preventDefault(); b.click(); }
});
term.write('\x1b[1;36mWelcome!\x1b[0m  Press \x1b[1;34m[\x1b[37mP\x1b[34m]\x1b[0m to play.\r\n');
