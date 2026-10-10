// Command sre-server puts Solar Realms Elite on the web, as a BBS would
// have put it on a phone line: one player at a time, each in a session
// of the game (the native sre binary) on its own terminal.
//
// Players are known by a public key their browser keeps.  The first time
// a key connects, the player picks a name (first come, first served).
// The page connects over a WebSocket, proves it holds the key by signing
// a challenge, and then the WebSocket carries the terminal.
package main

import (
	"context"
	"crypto/ecdsa"
	"crypto/rand"
	"crypto/sha256"
	"crypto/x509"
	"embed"
	"encoding/base64"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io/fs"
	"log"
	"math/big"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/coder/websocket"
	"github.com/creack/pty"
)

//go:embed web
var webFiles embed.FS

var (
	listen  = flag.String("listen", ":8000", "address to serve on")
	gameDir = flag.String("game", "game", "the game's directory")
	sreBin  = flag.String("sre", "sre", "the game's binary")
	keyFile = flag.String("keys", "keys.json", "the registered players' keys")
	sysops  = flag.String("sysops", "", "comma-separated names that may use the sysop menu")
)

func main() {
	flag.Parse()
	bin, err := exec.LookPath(*sreBin)
	if err != nil {
		log.Fatalf("sre binary: %v", err)
	}
	*sreBin, _ = filepath.Abs(bin)
	reg, err := loadRegistry(*keyFile)
	if err != nil {
		log.Fatalf("keys: %v", err)
	}
	srv := &server{reg: reg, line: make(chan struct{}, 1)}

	static, _ := fs.Sub(webFiles, "web")
	http.Handle("/", http.FileServer(http.FS(static)))
	http.HandleFunc("/session", srv.session)
	log.Printf("serving on %s, game in %s", *listen, *gameDir)
	log.Fatal(http.ListenAndServe(*listen, nil))
}

// ---------------------------------------------------------------------
// Players: public key fingerprints and their names.

type registry struct {
	mu    sync.Mutex
	path  string
	Names map[string]string `json:"names"` // key fingerprint -> name
}

func loadRegistry(path string) (*registry, error) {
	r := &registry{path: path, Names: map[string]string{}}
	b, err := os.ReadFile(path)
	if errors.Is(err, os.ErrNotExist) {
		return r, nil
	}
	if err != nil {
		return nil, err
	}
	return r, json.Unmarshal(b, r)
}

func (r *registry) lookup(fp string) string {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.Names[fp]
}

// register gives fp the name, if nobody has it.
func (r *registry) register(fp, name string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	for _, n := range r.Names {
		if strings.EqualFold(n, name) {
			return fmt.Errorf("%q is taken", name)
		}
	}
	r.Names[fp] = name
	b, err := json.MarshalIndent(r, "", "  ")
	if err != nil {
		return err
	}
	tmp := r.path + ".tmp"
	if err := os.WriteFile(tmp, b, 0o600); err != nil {
		return err
	}
	return os.Rename(tmp, r.path)
}

// validName: what the game can take as a BBS user's name.
func validName(s string) bool {
	if s == "" || len(s) > 30 || strings.TrimSpace(s) != s {
		return false
	}
	for _, c := range s {
		if c < 32 || c > 126 {
			return false
		}
	}
	return true
}

// ---------------------------------------------------------------------
// The WebSocket protocol.  Text messages are JSON control messages;
// binary messages are the terminal's bytes.
//
//	server: {"type":"challenge","nonce":"<base64>"}
//	page:   {"type":"auth","key":"<base64 SPKI>","sig":"<base64 r||s>",
//	         "mode":"play"|"sysop","rows":25}
//	server: {"type":"hello","name":"..."}  (or asks for a name in the terminal)

type authMsg struct {
	Type string `json:"type"`
	Key  string `json:"key"`
	Sig  string `json:"sig"`
	Mode string `json:"mode"`
	Rows int    `json:"rows"`
}

type server struct {
	reg  *registry
	line chan struct{} // the phone line: one player at a time

	mu      sync.Mutex
	playing string // who has the line
}

func (s *server) session(w http.ResponseWriter, r *http.Request) {
	ws, err := websocket.Accept(w, r, nil)
	if err != nil {
		return
	}
	defer ws.CloseNow()
	ctx := r.Context()
	c := &conn{ws: ws, ctx: ctx}

	nonce := make([]byte, 32)
	rand.Read(nonce)
	c.control(map[string]string{"type": "challenge", "nonce": base64.StdEncoding.EncodeToString(nonce)})
	var a authMsg
	if err := c.readJSON(&a); err != nil || a.Type != "auth" {
		return
	}
	fp, err := verify(a.Key, a.Sig, nonce)
	if err != nil {
		c.say("\r\nSorry, your key could not be checked.\r\n")
		ws.Close(websocket.StatusPolicyViolation, "bad key")
		return
	}
	c.start()
	name := s.reg.lookup(fp)
	if name == "" {
		if name, err = s.newPlayer(c, fp); err != nil {
			return
		}
	}
	c.control(map[string]string{"type": "hello", "name": name})
	log.Printf("%s connected (%s)", name, a.Mode)

	args := []string{}
	if a.Mode == "sysop" {
		if !isSysop(name) {
			c.say("\r\nOnly the sysop can do that.\r\n")
			ws.Close(websocket.StatusNormalClosure, "")
			return
		}
		args = []string{"reset"} // sets up the game if need be, then the sysop menu
	}
	if !s.takeLine(c, name) {
		return
	}
	defer s.hangUp()
	rows := a.Rows
	if rows < 10 || rows > 100 {
		rows = 25
	}
	if err := s.play(c, name, rows, args); err != nil {
		log.Printf("%s: %v", name, err)
	}
	ws.Close(websocket.StatusNormalClosure, "")
	log.Printf("%s hung up", name)
}

func isSysop(name string) bool {
	for _, n := range strings.Split(*sysops, ",") {
		if n = strings.TrimSpace(n); n != "" && strings.EqualFold(n, name) {
			return true
		}
	}
	return false
}

// verify checks sig, an ECDSA P-256 signature (WebCrypto's r||s) of
// nonce by key (SPKI), and returns the key's fingerprint.
func verify(key, sig string, nonce []byte) (string, error) {
	der, err := base64.StdEncoding.DecodeString(key)
	if err != nil {
		return "", err
	}
	raw, err := base64.StdEncoding.DecodeString(sig)
	if err != nil || len(raw) != 64 {
		return "", errors.New("bad signature")
	}
	pub, err := x509.ParsePKIXPublicKey(der)
	if err != nil {
		return "", err
	}
	ec, ok := pub.(*ecdsa.PublicKey)
	if !ok {
		return "", errors.New("not an ECDSA key")
	}
	h := sha256.Sum256(nonce)
	rr := new(big.Int).SetBytes(raw[:32])
	ss := new(big.Int).SetBytes(raw[32:])
	if !ecdsa.Verify(ec, h[:], rr, ss) {
		return "", errors.New("bad signature")
	}
	fp := sha256.Sum256(der)
	return base64.RawStdEncoding.EncodeToString(fp[:]), nil
}

// newPlayer asks a new key's holder for a name.
func (s *server) newPlayer(c *conn, fp string) (string, error) {
	c.say("\r\nWelcome, stranger!  This browser holds a new key, so you are a new\r\n" +
		"player.  The name you choose is yours from now on, on this browser.\r\n")
	for {
		c.say("\r\nWhat name would you like?  ")
		name, err := c.readLine(30)
		if err != nil {
			return "", err
		}
		if !validName(name) {
			c.say("\r\nNames are 1 to 30 letters, digits, spaces or punctuation.\r\n")
			continue
		}
		if err := s.reg.register(fp, name); err != nil {
			c.say("\r\nSorry, that name is taken.\r\n")
			continue
		}
		log.Printf("new player %s", name)
		return name, nil
	}
}

// takeLine waits for the line, telling the player who has it.
func (s *server) takeLine(c *conn, name string) bool {
	select {
	case s.line <- struct{}{}:
	default:
		s.mu.Lock()
		who := s.playing
		s.mu.Unlock()
		c.say(fmt.Sprintf("\r\n%s is playing right now.  Please wait; you're next in line...\r\n", who))
	wait:
		for {
			select {
			case s.line <- struct{}{}:
				break wait
			case <-c.keys: // typed while waiting: ignored
			case <-c.closed:
				return false
			}
		}
		c.say("\r\nIt's your turn!\r\n")
	}
	s.mu.Lock()
	s.playing = name
	s.mu.Unlock()
	return true
}

func (s *server) hangUp() {
	s.mu.Lock()
	s.playing = ""
	s.mu.Unlock()
	<-s.line
}

// play runs a session of the game on a terminal and connects it to the
// WebSocket until one of them ends.
func (s *server) play(c *conn, name string, rows int, args []string) error {
	door := strings.Join([]string{name, "1", "1", fmt.Sprint(rows), "38400", "1", "60", name}, "\r\n") + "\r\n"
	if err := os.WriteFile(filepath.Join(*gameDir, "DOORFILE.SR"), []byte(door), 0o644); err != nil {
		return err
	}
	cmd := exec.Command(*sreBin, args...)
	cmd.Dir = *gameDir
	cmd.Env = append(os.Environ(), "SRE_CHARSET=utf8", "TERM=xterm")
	tty, err := pty.StartWithSize(cmd, &pty.Winsize{Rows: uint16(rows), Cols: 80})
	if err != nil {
		return err
	}
	done := make(chan struct{})
	go func() { // game -> player
		buf := make([]byte, 4096)
		for {
			n, err := tty.Read(buf)
			if n > 0 {
				if c.ws.Write(c.ctx, websocket.MessageBinary, buf[:n]) != nil {
					break
				}
			}
			if err != nil {
				break
			}
		}
		close(done)
	}()
	go func() { // player -> game
	loop:
		for {
			select {
			case b := <-c.keys:
				tty.Write(b)
			case <-c.closed:
				break loop
			case <-done:
				return
			}
		}
		// The player left: hang up, as a modem would.  The game saves and
		// exits; make sure of it after a while.
		cmd.Process.Signal(syscall.SIGHUP)
		time.AfterFunc(30*time.Second, func() { cmd.Process.Kill() })
	}()
	err = cmd.Wait()
	tty.Close()
	<-done
	return err
}

// ---------------------------------------------------------------------

type conn struct {
	ws     *websocket.Conn
	ctx    context.Context
	keys   chan []byte   // what the player types
	closed chan struct{} // closed when the player has gone
}

// start reads the player's keys from here on.
func (c *conn) start() {
	c.keys = make(chan []byte, 64)
	c.closed = make(chan struct{})
	go func() {
		defer close(c.closed)
		for {
			typ, b, err := c.ws.Read(c.ctx)
			if err != nil {
				return
			}
			if typ == websocket.MessageBinary {
				select {
				case c.keys <- b:
				case <-c.ctx.Done():
					return
				}
			}
		}
	}()
}

func (c *conn) control(v any) {
	b, _ := json.Marshal(v)
	c.ws.Write(c.ctx, websocket.MessageText, b)
}

func (c *conn) say(s string) {
	c.ws.Write(c.ctx, websocket.MessageBinary, []byte(s))
}

func (c *conn) readJSON(v any) error {
	ctx, cancel := context.WithTimeout(c.ctx, 30*time.Second)
	defer cancel()
	typ, b, err := c.ws.Read(ctx)
	if err != nil {
		return err
	}
	if typ != websocket.MessageText {
		return errors.New("expected a control message")
	}
	return json.Unmarshal(b, v)
}

// readLine reads a line typed on the terminal, echoing it.
func (c *conn) readLine(max int) (string, error) {
	var line []byte
	for {
		var b []byte
		select {
		case b = <-c.keys:
		case <-c.closed:
			return "", errors.New("gone")
		}
		for _, ch := range b {
			switch {
			case ch == '\r' || ch == '\n':
				c.say("\r\n")
				return string(line), nil
			case ch == 8 || ch == 127:
				if len(line) > 0 {
					line = line[:len(line)-1]
					c.say("\b \b")
				}
			case ch >= 32 && ch < 127 && len(line) < max:
				line = append(line, ch)
				c.say(string(ch))
			}
		}
	}
}
