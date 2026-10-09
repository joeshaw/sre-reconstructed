// Terminal devices of the SRCOMM door library (segment 247e).
#include "devices.h"
#include "resource.h"
#include "brtl.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <termios.h>
#include <poll.h>
#include <errno.h>

int16 g_timeout = 60;
int16 g_lastremote = 0;
int16 g_ansi = 0;
int16 g_remote = 0;
int16 g_nonstop = 0;
int16 g_ibm = 1;
int16 g_effects = 0;
int16 g_linecount = 0;
int16 g_pagelen = 0x15;
char g_breakch = 3;
char g_stopch = 4;
Color c_normal(7, 0);
Color c_input(15, -1);
uint8 g_sbback = 0;
uint8 g_sbmid = 7;
uint8 g_sbfore = 15;
int16 g_pushback = 0;

KeyIn kin;
MultiOut kout;
LocalOut klocal;
int16 g_console = 0;
ModemOut kmodem;
OFStream klog;
static Resource r_userbreak("UserBreak", 1);

// ---------------------------------------------------------------------
// Host terminal: stdout/stdin stand in for the FOSSIL driver.

static struct termios g_saved;
static int g_rawmode = 0;
static char g_outbuf[4096];
static int g_outlen = 0;
static int g_peek = -2;

void TermRestore()
{
    TermFlush();
    if (g_rawmode) {
        tcsetattr(0, TCSANOW, &g_saved);
        g_rawmode = 0;
    }
}

void TermInit()
{
    atexit(TermFlush);
    if (isatty(0) && tcgetattr(0, &g_saved) == 0) {
        struct termios t = g_saved;
        t.c_iflag &= ~(ICRNL | INLCR | IXON | ISTRIP);
        t.c_lflag &= ~(ICANON | ECHO | ISIG | IEXTEN);
        t.c_cc[VMIN] = 1;
        t.c_cc[VTIME] = 0;
        tcsetattr(0, TCSANOW, &t);
        g_rawmode = 1;
        atexit(TermRestore);
    }
}

void TermFlush()
{
    int off = 0;
    while (off < g_outlen) {
        ssize_t r = write(1, g_outbuf + off, g_outlen - off);
        if (r <= 0) {
            if (r < 0 && errno == EINTR)
                continue;
            break;
        }
        off += (int)r;
    }
    g_outlen = 0;
}

void TermWrite(const char *s, int n)
{
    for (int i = 0; i < n; i++) {
        if (g_outlen == (int)sizeof(g_outbuf))
            TermFlush();
        g_outbuf[g_outlen++] = s[i];
    }
}

// 1 if a character (or end of input) is waiting.
int TermReady()
{
    if (g_peek != -2)
        return 1;
    TermFlush();
    struct pollfd p;
    p.fd = 0;
    p.events = POLLIN;
    p.revents = 0;
    if (poll(&p, 1, 0) > 0)
        return 1;
    return 0;
}

int TermGetc()
{
    if (g_peek != -2) {
        int c = g_peek;
        g_peek = -2;
        return c;
    }
    TermFlush();
    unsigned char c;
    for (;;) {
        ssize_t r = read(0, &c, 1);
        if (r == 1)
            return c;
        if (r < 0 && errno == EINTR)
            continue;
        return -1;
    }
}

// Carrier: lost when the user's input stream ends.
static int Carrier()
{
    if (g_peek == -1)
        return 0;
    if (TermReady() && g_peek == -2) {
        g_peek = TermGetc();
        if (g_peek == -1)
            return 0;
    }
    return 1;
}

// Test scripts (SRE_TESTMARK set) may hold back input with a NUL byte:
// what follows is only seen once the program waits for a key.
static int g_testmark = getenv("SRE_TESTMARK") != 0;
static int g_waiting;

static int RemoteCharReady()
{
    for (;;) {
        if (!TermReady())
            return 0;
        if (g_peek == -2)
            g_peek = TermGetc();
        if (g_peek == 0 && g_testmark) {
            if (!g_waiting)
                return 0;
            g_peek = -2;
            continue;
        }
        return g_peek >= 0;
    }
}

// The host terminal is the local keyboard when nobody is connected.
static int LocalKeyReady()
{
    return g_console && !g_remote && RemoteCharReady();
}

static int RemoteGetc()
{
    int c = TermGetc();
    return c < 0 ? 0 : c;
}

// ---------------------------------------------------------------------
// 247e:10a7
void ModemOut::RawWrite(const char *s, int16 n)
{
    if (!g_remote)
        return;
    char *conv = 0;
    if (!g_ibm) {
        conv = new char[n + 1];
        for (int16 i = 0; i < n; i++) {
            uint8 c = (uint8)s[i];
            if (c == 0xb3 || c == 0xba)
                c = '|';
            else if (c == 0xc4)
                c = '-';
            else if (c == 0xcd)
                c = '=';
            else if (c > 0x7f)
                c = '#';
            conv[i] = (char)c;
        }
    }
    if (g_peek == -1 && 0) {
        int16 port = g_remote;
        g_remote = 0;
        kout << "\a\n\xaeHANGUP:  COM" << port << ": could not write character\xaf\n";
        exit(2);
    }
    TermWrite(conv ? conv : s, n);
    delete[] conv;
}

// 247e:0fda: the original writes to its local screen window (or the raw
// screen in mode 1).
void LocalOut::RawWrite(const char *s, int16 n)
{
    if (g_console && !g_remote)
        TermWrite(s, n);
}

// ---------------------------------------------------------------------
// 247e:0c6a
MultiOut::MultiOut() : OStream(0x4f)
{
    logopen = 0;
    logon = 0;
    localon = 1;
    remoteon = 1;
    mode = 0;
}

// 247e:0cda
MultiOut::~MultiOut()
{
    if (logopen)
        CloseLog();
}

// 247e:0d2b
void MultiOut::EmitColor()
{
    curfg = newfg;
    curbg = newbg;
    if (!g_ansi)
        return;
    if (localon)
        klocal << Color(newfg, newbg);
    kmodem << Color(newfg, newbg);
}

// 247e:0db4
void MultiOut::OpenLog(const char *name)
{
    if (logopen)
        klog.close();
    logopen = 1;
    klog.open(name, 0);
    if (!g_ansi)
        klog.noansi = 1;
    logon = 1;
}

// 247e:0e07
void MultiOut::CloseLog()
{
    klog.close();
    logopen = 0;
    logon = 0;
}

// 247e:0e2a
void MultiOut::RawWrite(const char *s, int16 n)
{
    int16 pause = 0;
    if (!raw) {
        for (int16 i = 0; i < n; i++)
            if (s[i] == '\n')
                g_linecount++;
        if (g_linecount > g_pagelen && g_pagelen > 12) {
            g_linecount = 0;
            if (!g_nonstop)
                pause = 1;
        }
    }
    if (logon)
        klog.RawWrite(s, n);
    if (localon)
        klocal.Write(s, n);
    if (remoteon && g_remote)
        kmodem.Write(s, n);
    if (pause && !kin.disabled)
        Pause();
}

// ---------------------------------------------------------------------
// 247e:026b
void KeyIn::Echo(char c)
{
    kout << c;
}

// 247e:0285
void KeyIn::BeginInput()
{
    savefg = kout.newfg;
    savebg = kout.newbg;
    kout << c_input;
    if (c_input.bg <= 0)
        return;
    if (!g_ansi)
        return;
    int16 savedraw = kout.raw;
    int16 savedmode = kout.mode;
    kout.raw = 1;
    kout.SetMode(1);
    if (maxlen > 0) {
        for (int16 i = 0; i < maxlen; i++)
            kout << ' ';
        if (g_ansi) {
            kout << "\x1b[" << maxlen << 'D';
        } else {
            for (int16 i = 0; i < maxlen; i++)
                kout << '\b';
        }
    }
    if (!savedraw)
        kout.raw = 0;
    kout.SetMode(savedmode);
}

// 247e:039e
void KeyIn::EndInput()
{
    kout << Color(savefg, savebg);
}

// 247e:17dd / 247e:03ce: the function called while waiting for keys.
static void (*g_idle)() = 0;             // DS:bee0
void SetIdle(void (*fn)()) { g_idle = fn; }
static void Idle()
{
    if (g_idle)
        g_idle();
}

// 247e:03e9
int16 KeyIn::KeyReady()
{
    static int16 calls;                  // DS:bf0b
    if (calls++ == 10) {
        Idle();
        calls = 0;
    }
    // Carrier loss is detected by the blocking read in GetKey; here the
    // end of the input just means no key.
    if (g_remote && !Carrier())
        return 0;
    if (disabled)
        return 0;
    if (g_remote && RemoteCharReady())
        return 1;
    if (LocalKeyReady() && kout.localon)
        return 1;
    return 0;
}

// 247e:04d9
char KeyIn::GetKey()
{
    char cursor = g_ibm ? (char)0xdb : '_';
    const char *back = g_ansi ? "\x1b[D" : "\b";
    int16 savedraw = kout.raw;
    int16 savedmode = kout.mode;
    int16 savefg = kout.newfg;
    int16 savebg = kout.newbg;
    if (disabled)
        return 0;
    g_linecount = 0;
    if (g_pushback) {
        char c = (char)g_pushback;
        g_pushback = 0;
        return c;
    }
    kout.SetMode(1);
    if (!savedraw) {
        kout.raw = 1;
        kout << Color(savefg, 0) << cursor << back;
        kout.raw = 0;
    }
    int16 warned = 0;
    int32 start = b_time(0);
    int32 last = b_time(0);
    struct Waiting {
        Waiting() { g_waiting = 1; }
        ~Waiting() { g_waiting = 0; }
    } waiting;
    while (start + g_timeout + 30 > last) {
        if (g_remote && !Carrier()) {
            klocal << Color(14) << "Hangup?\n";
            exit(1);
        }
        if (g_console && !g_remote && !Carrier())
            exit(1);            // the end of the host's input
        if (KeyReady() && g_remote && RemoteCharReady()) {
            g_lastremote = 1;
            int16 c = (int8)RemoteGetc();
            if (!savedraw && c == 0x7f)
                c = 8;
            if (!savedraw)
                kout << " \b";
            const char *prompt = 0;
            if (c == g_breakch && r_userbreak.Bool())
                prompt = "\nBreak out of Program?  [y/N]";
            else if (c == g_stopch && r_userbreak.Bool())
                prompt = "\nStop Program?  [y/N]";
            if (!prompt) {
                kout.SetMode(savedmode);
                kout << Color(savefg, savebg);
                return (char)c;
            }
            AttrString saved(kout.line);
            int16 fg2 = kout.newfg;
            int16 bg2 = kout.newbg;
            kout << Color(7, 0) << prompt;
            for (;;) {
                char k;
                kin >> k;
                if (k == 'Y' || k == 'y') {
                    if (c == g_breakch)
                        Interrupt(2);
                    else
                        exit(0);
                }
                if (k == 'N' || k == 'n' || k == '\r')
                    break;
            }
            if (!savedraw)
                kout << Color(7) << saved << Color(fg2, bg2);
            if (!savedraw) {
                kout.raw = 1;
                kout << cursor << back;
                kout.raw = 0;
            }
            continue;
        }
        if (LocalKeyReady()) {
            // The local keyboard: no break prompt, and the host's DEL is
            // the PC's backspace.  (Extended keys only reached the
            // sysop's hot keys.)
            g_lastremote = 0;
            int16 c = (int8)RemoteGetc();
            if (c == 0x7f)
                c = 8;
            if (!savedraw)
                kout << " \b";
            if (c == g_breakch)
                Interrupt(2);
            else if (c == g_stopch)
                exit(0);
            else if (kout.localon) {
                kout.SetMode(savedmode);
                kout << Color(savefg, savebg);
                return (char)c;
            }
            if (!savedraw) {
                kout.raw = 1;
                kout << cursor << back;
                kout.raw = 0;
            }
            continue;
        }
        int32 now = b_time(0);
        if (now - last > 5)
            start += now - last;
        last = now;
        if ((int32)g_timeout <= last - start && !warned) {
            AttrString saved(kout.line);
            int16 fg2 = kout.newfg;
            int16 bg2 = kout.newbg;
            if (!savedraw) {
                kout << Color(14, 0)
                     << " \n\aWarning!  Program will abort in 30 seconds if you do not respond...\n";
                kout << Color(7) << saved << Color(fg2, bg2);
            } else {
                kout << "\a\a\a";
            }
            warned = 1;
        }
        Idle();
        TermFlush();
        if (!TermReady())
            b_delay(15);
    }
    int32 port = g_remote;
    g_remote = 0;
    kout << Color(14, 0) << " \a\n\xaeTIMEOUT:  COM" << port << ": no input for " << Color(15)
         << (int32)(last - start) << Color(14) << " seconds\xaf\n";
    exit(2);
    return 0;
}

// ---------------------------------------------------------------------
// 247e:11d7
void Pause()
{
    int16 spin = 0;
    char c;
    if (kin.disabled)
        return;
    int16 savedlog = kout.logon;
    kout << setw(0);
    if (kout.logopen)
        kout.logon = 0;
    int16 savedraw = kout.raw;
    kout.raw = 1;
    if (kin.KeyReady()) {
        kin >> c;
    } else {
        int16 savefg = kout.newfg;
        int16 savebg = kout.newbg;
        if (g_ibm) {
            kout << Color(g_sbback) << "\xb0\xb1\xb2\xdb" << Color(g_sbfore, g_sbback) << "PAUSED";
            kout << Color(g_sbback, 0) << "\xdb\xb2\xb1\xb0" << Color(13) << "  ";
        } else {
            kout << Color(g_sbback) << "\xc4\xc4" << Color(g_sbmid) << "\xc4\xc4" << Color(g_sbfore)
                 << "Paused" << Color(g_sbmid) << "\xc4\xc4" << Color(g_sbback) << "\xc4\xc4"
                 << Color(13) << "  ";
        }
        while (g_effects && !kin.KeyReady() && spin < 100) {
            if (g_ibm) {
                kout << Color(11) << "\b/";
                b_delay(50);
                kout << Color(1) << "\bX";
                b_delay(50);
                kout << Color(14) << "\b\\";
                b_delay(50);
                kout << Color(4) << "\bX";
            } else {
                kout << Color(14) << "\b.";
                b_delay(0x24);
                kout << Color(12) << "\bo";
                b_delay(50);
                kout << Color(4) << "\bO";
            }
            b_delay(50);
            spin++;
        }
        kout << "\b ";
        if (!kin.KeyReady())
            kout << Color(15);
        kin >> c;
        kout << Color(savefg, savebg);
        for (spin = 0; spin < 16; spin++)
            kout << "\b \b";
    }
    g_linecount = 0;
    if (!savedraw)
        kout.raw = 0;
    if (kout.logopen)
        kout.logon = savedlog;
}

// 247e:1627
void Interrupt(int16 code)
{
    g_pagelen = 0;
    kout << '\n' << Color(12, 0) << '[' << Color(14) << "INTERRUPT" << Color(12) << ']' << Color(7)
         << '\n';
    if (code != 999)
        exit(1);
}
