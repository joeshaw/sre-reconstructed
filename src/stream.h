// Stream classes of the SRCOMM door library (resident segment 229f).
//
// The original game writes all text through C++ streams ("kout << ...")
// with color manipulators.  The base output stream keeps the current
// line in an attributed buffer so that it can word-wrap at the stream's
// width, and it emits ANSI color sequences lazily, just before text.
#ifndef SRE_STREAM_H
#define SRE_STREAM_H

#include "types.h"

// A color manipulator: kout << Color(fg, bg).  bg == -1 keeps the
// current background.
struct Color {
    int16 fg;
    int16 bg;
    Color(int16 f = 7, int16 b = -1) : fg(f), bg(b) {}
};

// Line buffer with per-character colors (229f:0bea).
struct AttrString {
    char *buf;
    int16 *fg;
    int16 *bg;
    int16 len;
    int16 size;
    int16 maxsize;

    explicit AttrString(int16 n);
    AttrString(const AttrString &o);
    ~AttrString();
    AttrString &operator=(const AttrString &o);
    void append(char c, int16 f, int16 b);
    void pop();
    void setsize(int16 n);
    void clear();
    char charAt(int16 i) const;
    int16 fgAt(int16 i) const;
    int16 bgAt(int16 i) const;
};

class OStream {
public:
    AttrString line;
    int16 f14;
    int16 width;        // field width for the next string/number
    int16 raw;          // nonzero: no buffering / word-wrap
    int8 curfg, curbg;  // colors already sent
    int8 newfg, newbg;  // colors requested
    int16 noansi;       // nonzero: never emit color sequences

    explicit OStream(int16 wrapwidth);
    virtual ~OStream();

    virtual void SetColor(int16 bg, int16 fg);
    virtual void Begin() {}
    virtual void End() {}
    virtual void EmitColor();
    virtual void Write(const char *s, int16 n);
    virtual void RawWrite(const char *s, int16 n);
    virtual char PutRaw(char c) { return c; }

    OStream &operator<<(const char *s);
    OStream &operator<<(char c);
    OStream &operator<<(int32 v);
    OStream &operator<<(int16 v);
    OStream &operator<<(uint16 v);
    OStream &operator<<(const Color &c);
    OStream &operator<<(const AttrString &a);
    void PutNumber(int32 v);
};

// Manipulators that set the field width (229f:12ac, 229f:12c5).
struct SetW { int16 w; };
inline SetW setw(int16 w) { SetW s = { w }; return s; }
inline SetW setwr(int16 w) { SetW s = { (int16)-w }; return s; }
OStream &operator<<(OStream &o, SetW s);

// Translation hook used by "Output.TranslationFile" (DS:be63).
typedef const char *(*TranslateFn)(const char *);
TranslateFn SetTranslateHook(TranslateFn fn);

extern char g_comma;            // Output.Comma (DS:be62), 0 = none

class IStream {
public:
    int16 maxlen;       // max characters for line input
    int32 defval;       // default for numeric input, -1 = none
    int32 maxval;       // max for numeric input, -1 = none

    IStream();
    virtual ~IStream() {}
    virtual void Echo(char c) = 0;
    virtual int16 KeyReady() = 0;
    virtual char GetKey() = 0;
    virtual void BeginInput() {}
    virtual void EndInput() {}
    virtual void GetLine(char *buf);
    virtual int32 GetNumber(int32 max);

    IStream &operator>>(char &c);
    IStream &operator>>(char *s);
    IStream &operator>>(int32 &v);
    IStream &operator>>(int16 &v);
    IStream &operator>>(uint16 &v);
    IStream &operator>>(uint8 &v);
};

struct SetMaxLen { int16 n; };
struct SetDefault { int32 v; };
struct SetMax { int32 v; };
inline SetMaxLen maxlen(int16 n) { SetMaxLen m = { n }; return m; }
inline SetDefault defval(int32 v) { SetDefault d = { v }; return d; }
inline SetMax maxval(int32 v) { SetMax m = { v }; return m; }
IStream &operator>>(IStream &i, SetMaxLen m);
IStream &operator>>(IStream &i, SetDefault d);
IStream &operator>>(IStream &i, SetMax m);

#endif
