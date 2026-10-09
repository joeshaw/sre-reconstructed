// Stream classes of the SRCOMM door library (segment 229f).
#include "stream.h"
#include "brtl.h"
#include <string.h>
#include <stdlib.h>

char g_comma = ',';
static TranslateFn g_translate = 0;

// ---------------------------------------------------------------------
// _ctype (DS:c2ab).  Index -128..-1 are the bytes that precede the table
// in DGROUP at run time; a few original routines index with a
// sign-extended char and so see these for characters >= 0x80.
static const uint8 ctype_tab[256] = {
    32, 32, 32, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    1, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 64, 64, 64, 64, 64, 64,
    64, 20, 20, 20, 20, 20, 20, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 64, 64, 64, 64, 64,
    64, 24, 24, 24, 24, 24, 24, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 64, 64, 64, 64, 32,
};
static const uint8 ctype_before[128] = {
    78, 79, 0, 0, 82, 69, 83, 79, 85, 82, 67, 69, 46, 68, 65, 84,
    0, 82, 69, 83, 79, 85, 82, 67, 69, 46, 68, 69, 70, 0, 35, 69,
    79, 70, 0, 116, 114, 117, 101, 0, 116, 0, 111, 110, 0, 121, 101, 115,
    0, 200, 212, 85, 45, 216, 212, 85, 45, 184, 212, 85, 45, 0, 35, 60,
    101, 114, 114, 111, 114, 62, 0, 32, 41, 0, 35, 60, 103, 97, 114, 98,
    97, 103, 101, 62, 0, 47, 0, 12, 42, 82, 0, 12, 42, 154, 12, 12,
    42, 69, 0, 12, 42, 58, 0, 12, 42, 210, 12, 12, 42, 47, 0, 12,
    42, 58, 0, 12, 42, 6, 0, 12, 42, 0, 0, 0, 0, 0, 0, 0,
};

int ctype_of(uint8 c) { return ctype_tab[c]; }

int ctype_signed(int8 c)
{
    if (c >= 0)
        return ctype_tab[(uint8)c];
    return ctype_before[128 + c];
}

// ---------------------------------------------------------------------
// AttrString (229f:0bea .. 229f:10a5)

AttrString::AttrString(int16 n)
{
    if (n > 0) {
        buf = (char *)malloc(n + 2);
        fg = (int16 *)malloc((n + 2) * 2);
        bg = (int16 *)malloc((n + 2) * 2);
    } else {
        buf = 0;
        fg = 0;
        bg = 0;
    }
    size = n;
    maxsize = n;
    if (buf)
        buf[0] = 0;
    len = 0;
}

AttrString::AttrString(const AttrString &o) : buf(0), fg(0), bg(0)
{
    *this = o;
}

// 229f:0d5e
AttrString &AttrString::operator=(const AttrString &o)
{
    if (this == &o)
        return *this;
    free(buf);
    free(fg);
    free(bg);
    size = o.size;
    maxsize = o.size;
    if (size > 0) {
        buf = (char *)malloc(size + 2);
        fg = (int16 *)malloc((size + 2) * 2);
        bg = (int16 *)malloc((size + 2) * 2);
    } else {
        buf = 0;
        fg = 0;
        bg = 0;
    }
    if (buf) {
        strcpy(buf, o.buf);
        len = o.len;
    } else {
        len = 0;
    }
    if (fg)
        for (int16 i = 0; i < maxsize + 2; i++)
            fg[i] = o.fg[i];
    if (bg)
        for (int16 i = 0; i < maxsize + 2; i++)
            bg[i] = o.bg[i];
    return *this;
}

AttrString::~AttrString()
{
    free(buf);
    free(bg);
    free(fg);
}

void AttrString::append(char c, int16 f, int16 b)
{
    if (buf && len <= size) {
        fg[len] = f;
        bg[len] = b;
        buf[len++] = c;
        buf[len] = 0;
    }
}

void AttrString::pop()
{
    if (buf && len > 0)
        buf[--len] = 0;
}

void AttrString::setsize(int16 n)
{
    size = (maxsize < n) ? maxsize : n;
}

void AttrString::clear()
{
    len = 0;
    if (buf)
        buf[0] = 0;
}

char AttrString::charAt(int16 i) const
{
    if (buf && i >= 0 && size > i)
        return buf[i];
    return 0;
}

int16 AttrString::fgAt(int16 i) const
{
    if (fg && i >= 0 && size > i)
        return fg[i];
    return 0;
}

int16 AttrString::bgAt(int16 i) const
{
    if (bg && i >= 0 && size > i)
        return bg[i];
    return 0;
}

// ---------------------------------------------------------------------
// OStream

OStream::OStream(int16 wrapwidth) : line(wrapwidth)
{
    curfg = curbg = newfg = newbg = 0;
    width = 0;
    raw = 0;
    noansi = 0;
    f14 = 0;
}

OStream::~OStream() {}

// 229f:1d69
void OStream::SetColor(int16 bg, int16 fg)
{
    newfg = (int8)fg;
    if (bg != -1)
        newbg = (int8)bg;
    else
        newbg = curbg;
}

// 229f:1664: build "ESC[...m" for the change from cur* to new*.
void OStream::EmitColor()
{
    static const char ansi[] = "04261537";
    char seq[0x10];
    int16 n;

    if (noansi)
        return;
    n = 2;
    seq[0] = 0x1b;
    seq[1] = '[';
    if (newbg == -1)
        newbg = curbg;
    if ((curfg >= 8 && newfg < 8) || (newfg == 7 && newbg == 0)) {
        curfg = 7;
        curbg = 0;
        seq[n++] = '0';
        seq[n++] = ';';
    }
    if (curfg < 8 && newfg >= 8) {
        curfg |= 8;
        seq[n++] = '1';
        seq[n++] = ';';
    }
    if (curfg != newfg) {
        curfg = newfg;
        seq[n++] = '3';
        seq[n++] = ansi[newfg & 7];
        seq[n++] = ';';
    }
    if (curbg != newbg) {
        curbg = newbg;
        seq[n++] = '4';
        seq[n++] = ansi[newbg & 7];
        seq[n++] = ';';
    }
    seq[n] = 0;
    n--;
    seq[n] = 'm';
    if (n > 1)
        RawWrite(seq, n + 1);
}

// 229f:182e: buffered write with word wrap.
void OStream::Write(const char *s, int16 n)
{
    if (curfg != newfg || curbg != newbg)
        EmitColor();

    if (raw) {
        const char *seg = s;
        int16 seglen = 0;
        for (int16 i = 0; i < n; i++, seglen++) {
            if (s[i] == '\r') {
                RawWrite(seg, seglen);
                seglen = -1;
                seg = s + i + 1;
            } else if (s[i] == '\n') {
                RawWrite(seg, seglen);
                RawWrite("\r\n", 2);
                seglen = -1;
                seg = s + i + 1;
            }
        }
        RawWrite(seg, seglen);
        return;
    }

    const char *seg = s;
    int16 wrap = line.size;
    int16 seglen = 0;
    for (int16 i = 0; i < n; i++, seglen++) {
        uint8 c = (uint8)s[i];
        if (c == '\r') {
            RawWrite(seg, seglen);
            seglen = -1;
            seg = s + i + 1;
            continue;
        }
        if (c == '\n') {
            RawWrite(seg, seglen);
            RawWrite("\r\n", 2);
            seglen = -1;
            seg = s + i + 1;
            line.clear();
            continue;
        }
        if (c == 8) {
            line.pop();
            continue;
        }
        if (!(ctype_of(c) & CT_CTL))
            line.append((char)c, curfg, curbg);
        int16 pos = line.len;
        if (wrap <= 0 || pos < wrap)
            continue;

        // The line is full: look back for a space within the last
        // quarter of the line.
        int16 limit = (int16)(wrap * 3 / 4);
        while (!(ctype_of((uint8)line.charAt(pos)) & CT_SP) && pos > limit)
            pos--;
        RawWrite(seg, seglen + 1);
        seglen = -1;
        seg = s + i + 1;
        if (pos > limit) {
            // Erase the partial word, newline, and re-print it.
            pos++;
            int16 fg = newfg, bg = newbg;
            SetColor(0, fg);
            EmitColor();
            for (int16 j = pos; j < line.len; j++)
                RawWrite("\b \b", 3);
            RawWrite("\r\n", 2);
            SetColor(bg, fg);
            EmitColor();
            AttrString word(line);
            line.clear();
            for (; pos < word.len; pos++) {
                SetColor(word.bgAt(pos), word.fgAt(pos));
                char wc = word.charAt(pos);
                if (!(ctype_of((uint8)wc) & CT_CTL))
                    line.append(wc, curfg, curbg);
                RawWrite(&wc, 1);
            }
        } else {
            int16 fg = newfg, bg = newbg;
            SetColor(0, fg);
            EmitColor();
            line.clear();
            RawWrite("\r\n", 2);
            SetColor(bg, fg);
            EmitColor();
        }
    }
    RawWrite(seg, seglen);
}

// 229f:1d26
void OStream::RawWrite(const char *s, int16 n)
{
    for (int16 i = 0; i < n; i++)
        PutRaw(s[i]);
}

// 229f:12e0
TranslateFn SetTranslateHook(TranslateFn fn)
{
    TranslateFn old = g_translate;
    g_translate = fn;
    return old;
}

// 229f:130c
OStream &OStream::operator<<(const char *s)
{
    if (!s)
        return *this << "(NULL POINTER!!!)";
    if (g_translate)
        s = g_translate(s);
    TranslateFn saved = SetTranslateHook(0);
    Begin();
    char sp = ' ';
    int16 pad = (int16)(width + strlen(s));
    for (; pad < 0; pad++)
        Write(&sp, 1);
    pad = (int16)strlen(s);
    Write(s, pad);
    for (; width > pad; pad++)
        Write(&sp, 1);
    End();
    SetTranslateHook(saved);
    return *this;
}

// 229f:1267
OStream &OStream::operator<<(char c)
{
    Begin();
    Write(&c, 1);
    End();
    return *this;
}

// 229f:1425: decimal with thousands separators, right-justified in width.
void OStream::PutNumber(int32 v)
{
    char digits[0x52];
    int8 n = 0;
    int16 neg = 0;
    if (v < 0) {
        neg = -1;
        v = -v;
    }
    if (v == 0) {
        digits[0] = '0';
        n = 1;
    } else {
        while (v > 0) {
            digits[n] = (char)('0' + v % 10);
            n++;
            v /= 10;
            if (n % 4 == 3 && g_comma)
                digits[n++] = ',';
        }
        if (digits[n - 1] == ',' && g_comma) {
            n--;
            digits[n] = 0;
        }
        if (n == 5 && g_comma) {
            digits[3] = digits[4];
            digits[4] = 0;
            n--;
        }
    }
    if (neg)
        digits[n++] = '-';
    Begin();
    int8 w = (int8)width;
    if (w < 0)
        w = (int8)-w;
    char sp = ' ';
    for (; w > n; w--)
        Write(&sp, 1);
    while (--n >= 0) {
        if (digits[n] == ',')
            Write(&g_comma, 1);
        else
            Write(&digits[n], 1);
    }
    End();
}

OStream &OStream::operator<<(int32 v)
{
    PutNumber(v);
    return *this;
}

OStream &OStream::operator<<(int16 v)
{
    PutNumber((int32)v);
    return *this;
}

OStream &OStream::operator<<(uint16 v)
{
    PutNumber((int32)(uint32)v);
    return *this;
}

// 229f:1dd4
OStream &OStream::operator<<(const Color &c)
{
    SetColor(c.bg, c.fg);
    return *this;
}

// 229f:10e0
OStream &OStream::operator<<(const AttrString &a)
{
    if (!a.buf)
        return *this;
    for (int16 i = 0; i < a.len; i++) {
        if (a.fg && a.bg)
            *this << Color(a.fg[i], a.bg[i]);
        if (a.buf)
            *this << a.buf[i];
    }
    return *this;
}

OStream &operator<<(OStream &o, SetW s)
{
    o.width = s.w;
    return o;
}

// ---------------------------------------------------------------------
// IStream (229f:0008 .. 229f:0bea)

IStream::IStream()
{
    maxlen = 0;
    defval = -1;
    maxval = -1;
}

IStream &operator>>(IStream &i, SetMaxLen m)
{
    i.maxlen = m.n;
    return i;
}

IStream &operator>>(IStream &i, SetDefault d)
{
    i.defval = d.v;
    return i;
}

IStream &operator>>(IStream &i, SetMax m)
{
    i.maxval = m.v;
    return i;
}

// 229f:006c: line input; buf may hold a default that is shown first and
// replaced if the first key typed is printable.
void IStream::GetLine(char *buf)
{
    int8 i = 0;
    int16 first = 1;
    char c;

    BeginInput();
    while (buf[i]) {
        Echo(buf[i]);
        i++;
    }
    for (;;) {
        c = GetKey();
        if (c == 9)
            c = ' ';
        if (first && (uint8)c >= 0x20 && c != 0x7f) {
            for (; i > 0; i--) {
                Echo(8);
                Echo(' ');
                Echo(8);
            }
            buf[0] = 0;
        }
        first = 0;
        if (c == 8) {
            if (i > 0) {
                i--;
                buf[i] = 0;
                Echo(8);
                Echo(' ');
                Echo(8);
            }
        } else if ((uint8)c >= 0x20 && (uint8)c <= 0xfe && c != 0x7f) {
            if (i < maxlen || maxlen == 0) {
                buf[i] = c;
                i++;
                buf[i] = 0;
                Echo(c);
            }
        }
        if (c == '\n' || c == '\r' || c == 0)
            break;
    }
    Echo('\n');
    EndInput();
}

// 229f:02ec: numeric input with default, maximum, "%", ">" and K/M.
int32 IStream::GetNumber(int32 max)
{
    int8 i = 0;
    int32 v = -1;
    int8 digitsTyped = 0;
    int16 savedmax = maxlen;
    char pending[0x5e];
    int16 first;
    uint8 c;

    if (!(maxval >= max) && maxval >= 0)
        max = maxval;
    {
        int32 t = max;
        maxlen = 1;
        if (t == 0)
            maxlen = 2;
        while (t > 0) {
            maxlen++;
            t /= 10;
        }
    }
    BeginInput();
    maxlen = savedmax;
    pending[0] = 0;
    if (defval >= 0)
        b_ltoa(defval, pending, 10);
    first = 1;
    do {
        if (pending[0] == 0) {
            c = (uint8)b_toupper((int)(int8)GetKey());
            if (first && (ctype_signed((int8)c) & (CT_DIG | CT_UPP | CT_LOW))) {
                first = 0;
                int16 j;
                for (j = 0; j < i; j++)
                    pending[j] = 8;
                pending[j] = (char)c;
                pending[j + 1] = 0;
                goto next;
            }
            first = 0;
        } else {
            c = (uint8)pending[0];
            int16 j;
            for (j = 0; pending[j]; j++)
                pending[j] = pending[j + 1];
        }

        if (c == 8 && i > 0) {
            if (v > 9)
                v /= 10;
            else
                v = -1;
            Echo(8);
            Echo(' ');
            Echo(8);
            i--;
            digitsTyped = 1;
        } else if (c == '%' && v >= 0 && v <= 100 && maxval >= 0) {
            int32 r;
            for (; i > 0; i--) {
                Echo(8);
                Echo(' ');
                Echo(8);
            }
            if (maxval > 10000000L)
                r = (maxval / 100) * v;
            else
                r = (v * maxval) / 100;
            if (!(maxval >= r))
                r = maxval;
            v = r;
            i = 0;
            for (int32 t = v; t > 0; t /= 10)
                i++;
            for (int16 k = i - 1; k >= 0; k--) {
                int32 t = r;
                for (int16 d = 0; d < k; d++)
                    t /= 10;
                Echo((char)('0' + t % 10));
            }
            digitsTyped = 1;
        } else if (c >= '0' && c <= '9') {
            int32 t;
            if (v == -1) {
                t = (int8)c - '0';
                if (t > max)
                    t = -1;
                if (t >= 0) {
                    i++;
                    Echo((char)c);
                    v = t;
                }
            } else {
                if (v == 0) {
                    Echo(8);
                    i--;
                }
                t = v * 10 + ((int8)c - '0');
                if (t > max)
                    t = -1;
                else if (t / 10 != v)
                    t = -1;
                if (t >= 0) {
                    i++;
                    Echo((char)c);
                    v = t;
                } else if (v == 0) {
                    v = -1;
                }
            }
            digitsTyped = 1;
        } else if (c == '>' && maxval >= 0) {
            for (; i > 0; i--) {
                Echo(8);
                Echo(' ');
                Echo(8);
            }
            v = maxval;
            i = 0;
            for (int32 t = v; t > 0; t /= 10)
                i++;
            for (int16 k = i - 1; k >= 0; k--) {
                int32 t = maxval;
                for (int16 d = 0; d < k; d++)
                    t /= 10;
                Echo((char)('0' + t % 10));
            }
            v = maxval;
            digitsTyped = 1;
        } else if (c == 'K' || c == 'M') {
            if (digitsTyped) {
                if (c == 'K')
                    strcat(pending, "000");
                if (c == 'M')
                    strcat(pending, "000000");
                if (c == 'G')
                    strcat(pending, "000000000");
            }
        } else {
            digitsTyped = 0;
        }
    next:;
    } while (c != '\n' && c != '\r' && c != 0);

    if (i == 0)
        v = -1;
    if (v == -1) {
        char tmp[0x28];
        if (defval == -1)
            tmp[0] = 0;
        else
            b_ltoa(defval, tmp, 10);
        for (int16 k = 0; tmp[k]; k++)
            Echo(tmp[k]);
        v = defval;
    }
    defval = -1;
    maxval = -1;
    EndInput();
    Echo('\n');
    return v;
}

// 229f:029d
IStream &IStream::operator>>(char &c)
{
    c = GetKey();
    return *this;
}

// 229f:02c1
IStream &IStream::operator>>(char *s)
{
    s[0] = 0;
    GetLine(s);
    return *this;
}

// 229f:0ad7
IStream &IStream::operator>>(int32 &v)
{
    v = GetNumber(0x7fffffffL);
    return *this;
}

// 229f:0b18
IStream &IStream::operator>>(int16 &v)
{
    int32 r = GetNumber(0x7fffL);
    v = (r == -1) ? -1 : (int16)r;
    return *this;
}

// 229f:0b73
IStream &IStream::operator>>(uint16 &v)
{
    v = (uint16)GetNumber(0xffffL);
    return *this;
}

// 229f:0baf
IStream &IStream::operator>>(uint8 &v)
{
    v = (uint8)GetNumber(0xffL);
    return *this;
}
