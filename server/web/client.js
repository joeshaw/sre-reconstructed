// The page's side of sre-server: a key that identifies the player, kept
// in this browser, and a terminal connected to a session of the game.
'use strict';

const term = new Terminal({ cols: 80, rows: 25 });
term.open(document.getElementById('term'));
const status = (s) => { document.getElementById('status').textContent = s; };

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
  status('connecting');
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
    } else if (msg.type === 'hello') {
      status('connected as ' + msg.name);
    }
  };
  ws.onclose = () => {
    ws = null;
    status('disconnected');
    term.write('\r\n\x1b[0m[Disconnected]\r\n');
  };
}

document.getElementById('play').onclick = () => connect('play');
document.getElementById('sysop').onclick = () => connect('sysop');
status('press Play to connect');
