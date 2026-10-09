// Terminal devices of the SRCOMM door library (segment 247e).
//
// kout is a multiplexing output stream: text is word-wrapped once (at 79
// columns) and the resulting bytes are copied to the sysop's screen, the
// remote user's modem and an optional log file.  In this reconstruction
// the "modem" is the user's terminal (stdout/stdin).  Without a remote
// user (local play and the sysop functions) the terminal is the sysop's
// local screen and keyboard instead.
#ifndef SRE_DEVICES_H
#define SRE_DEVICES_H

#include "stream.h"
#include "files.h"

// Remote (modem) output stream (vtable c07d).  Receives the bytes kout
// has already wrapped; converts IBM box characters when the user has no
// IBM character set.
class ModemOut : public OStream {
public:
    ModemOut() : OStream(0) {}
    void RawWrite(const char *s, int16 n);      // 247e:10a7
};

// The sysop's local screen (vtable c099).  It is the host terminal when
// nobody is connected remotely (g_console), and discarded otherwise.
class LocalOut : public OStream {
public:
    LocalOut() : OStream(0) {}
    void RawWrite(const char *s, int16 n);      // 247e:0fda
};

// kout (vtable c0b5, DS:d3b8).
class MultiOut : public OStream {
public:
    int16 logopen;      // +20 (d3d8)
    int16 logon;        // +22 (d3da) copy output to the log
    int16 mode;         // +24 (d3dc)
    int16 localon;      // +26 (d3de) copy output to the local screen
    int16 remoteon;     // +28 (d3e0)
    MultiOut();
    ~MultiOut();
    void EmitColor();                            // 247e:0d2b
    void RawWrite(const char *s, int16 n);      // 247e:0e2a
    void OpenLog(const char *name);             // 247e:0db4
    void CloseLog();                            // 247e:0e07
    int16 SetMode(int16 m) { mode = m; return m; }  // 247e:0eff
};

// kin (vtable c0d1, DS:d3a6).
class KeyIn : public IStream {
public:
    int16 disabled;     // +10 (d3b6)
    int16 savefg;       // +0c
    int16 savebg;       // +0e
    KeyIn() : disabled(0), savefg(7), savebg(0) {}
    void Echo(char c);                          // 247e:026b
    int16 KeyReady();                           // 247e:03e9
    char GetKey();                              // 247e:04d9
    void BeginInput();                          // 247e:0285
    void EndInput();                            // 247e:039e
};

extern MultiOut kout;           // DS:d3b8
extern KeyIn kin;               // DS:d3a6
extern ModemOut kmodem;         // DS:d402
extern OFStream klog;           // DS:d422
extern LocalOut klocal;         // DS:d3e2
extern int16 g_console;         // the host terminal is the local console

extern int16 g_timeout;         // bee4 input timeout in seconds
extern int16 g_lastremote;      // bee6
extern int16 g_ansi;            // bee8
extern int16 g_remote;          // beea com port, 0 = local
extern int16 g_nonstop;         // beec
extern int16 g_ibm;             // beee
extern int16 g_effects;         // bef0 SpecialEffects
extern int16 g_linecount;       // bef2
extern int16 g_pagelen;         // bef4
extern char g_breakch;          // bef6
extern char g_stopch;           // bef7
extern Color c_normal;          // bef8
extern Color c_input;           // befc
extern uint8 g_sbback;          // bf04 status bar background
extern uint8 g_sbmid;           // bf05
extern uint8 g_sbfore;          // bf06
extern int16 g_pushback;        // bf19

// 247e:11d7: "PAUSED" prompt; waits for a key.
void Pause();
// 247e:1627: interrupt (Ctrl-Break) handler.
void Interrupt(int16 code);

// Host terminal.
void TermInit();
void TermFlush();
void TermWrite(const char *s, int n);
int TermReady();
int TermGetc();         // -1 at end of input
int TermPeek();         // the next TermGetc without taking it; -2 if none yet
void TermRestore();

// 247e:17dd: set the function called while waiting for keys; the old one.
typedef void (*IdleFn)();
IdleFn SetIdle(IdleFn fn);

#endif
