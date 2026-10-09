// News and the sysop's log (segment 5b6a).
#include "sre.h"

Color c_white(15);              // 6782
Color c_6792(4), c_6796(4), c_679a(1), c_679e(4), c_67a2(8);

// The news stream (vtable 683f, DS:d138): each line written becomes a
// news item in NEWS/NEWS0.MSG, with colours as "^\BF" codes.
class NewsStream : public OStream {
public:
    String line;        // +20
    NewsStream() : OStream(0) {}
    void EmitColor();
    char PutRaw(char c);
};

// The sysop's log (vtable 685f, DS:d160): lines go to SYSOP.LOG.
class SysopLog : public OStream {
public:
    AttrString text;    // +20
    SysopLog() : OStream(0xff), text(0xff)
    {
        noansi = 1;
        curfg = 0xe;
    }
    char PutRaw(char c);
};

static NewsStream g_news;
static SysopLog g_sysoplog;
OStream &g_log2 = g_news;
OStream &g_errlog = g_sysoplog;

// 5b6a:0b22
void NewsStream::EmitColor()
{
    static const char HEX[] = "0123456789ABCDEF";
    curfg = newfg;
    curbg = newbg;
    PutRaw('^');
    PutRaw('\\');
    PutRaw(HEX[(int)newbg]);
    PutRaw(HEX[(int)newfg]);
}

// 5b6a:04d6: length without colour codes.
static int16 VisibleLen(const String &s)
{
    int16 n = 0;
    for (int16 i = 0; i < s.length(); i++) {
        if (s.at(i) == '^' && s.at(i + 1) == '\\' && s.at(i + 2) && s.at(i + 3))
            i += 3;
        else
            n++;
    }
    return n;
}

// 5b6a:055a: split s at a space so that the first part is at most width
// visible characters; a short last part is padded to width.
static void SplitLine(const String &s, String &a, String &b, int16 width)
{
    int16 brk = -1;
    int16 vis = 0;
    for (int16 i = 0; i < s.length(); i++) {
        if ((ctype_signed(s.at(i)) & CT_SP) && i >= width / 2)
            brk = i;
        if (s.at(i) == '^' && s.at(i + 1) == '\\' && s.at(i + 2) && s.at(i + 3)) {
            if (i >= width / 2)
                brk = i;
            i += 3;
            continue;
        }
        if (vis >= width) {
            if (brk == -1)
                brk = vis;
            a = s.substr(0, brk);
            while (ctype_signed(s.at(brk)) & CT_SP)
                brk++;
            b = s.substr(brk, -1);
            return;
        }
        vis++;
    }
    a = s;
    for (; vis < width; vis++)
        a += ' ';
    b = "";
}

// 5b6a:0705
static void FormatNews(String &s, String first, String cont, String eol1, String eol2)
{
    String *eol = &eol1;
    s = first + s;
    String out;
    for (;;) {
        String a, b;
        SplitLine(s, a, b, (int16)(0x4b - VisibleLen(*eol)));
        out += a + *eol + "\n";
        eol = &eol2;
        if (b.length() == 0)
            break;
        s = cont + b;
    }
    s = out;
}

// 5b6a:0929
char NewsStream::PutRaw(char c)
{
    if (c == '\r')
        return c;
    if (c != '\n') {
        line += c;
        return c;
    }
    if (line.at(0) == '\\') {
        line.erase(0, 1);
    } else if (line.at(0) == '^' && line.at(1) == '\\' && line.at(4) == '\\') {
        line.erase(4, 1);
    } else if (line.length() > 0) {
        FormatNews(line, "^\\01\xc4^\\09\xc4^\\03\xc4^\\0B\xc4^\\07", "^\\01 \xfa  ^\\07", "", "");
    }
    line.insert(line.length(), String("\n"));
    OFStream f("NEWS/NEWS0.MSG", 1, 0, 0);
    int32 now = b_time(0);
    char comma = g_comma;
    g_comma = 0;
    f << "#date " << now << '\n';
    g_comma = comma;
    f << line.c_str();
    line = "";
    return c;
}

// 5b6a:0cf7
char SysopLog::PutRaw(char c)
{
    curfg = 0xe;
    if (c == '\n') {
        text = line;
        OFStream f("SYSOP.LOG", 1, 0, 0);
        f.noansi = 1;
        f << text << '\n';
    }
    return c;
}

// 5b6a:0000: show a news file from the given date on.
int16 ShowNews(OStream &o, IStream *in, const String &file, int32 since)
{
    IFStream f(file.c_str(), 0);
    if (f.fail())
        return 0;
    int16 show = 1;
    String line;
    o << Color(7);
    while (!f.fail()) {
        ReadLine(f, line);
        if (in && in->KeyReady()) {
            char c;
            *in >> c;
            if (c == ' ')
                break;
        }
        if (line.substr(0, 5) == String("#date")) {
            line.erase(0, 6);
            int32 t = StrToLong(line);
            show = t >= since;
            continue;
        }
        if (show) {
            PutCoded(line, o);
            o << "\n";
        }
    }
    o << Color(7);
    return 1;
}

// 5b6a:01fb
int16 ShowNews(const String &file, int32 since)
{
    return ShowNews(kout, &kin, file, since);
}

// 5b6a:0228: copy news file n into the files named by News<n>.File.*.
void News(int16 n)
{
    String digit(1, (char)(n + '0'));
    String msg = String("News/News") + digit + ".MSG";
    String ansikey = String("News") + digit + ".File.ANSI";
    String textkey = String("News") + digit + ".File.Text";
    Resource ra(ansikey.c_str(), "");
    Resource rt(textkey.c_str(), "");
    if (ra.Str()[0]) {
        OFStream f(ra.Str(), 0);
        f.noansi = 0;
        ShowNews(f, 0, msg, 0);
    }
    if (rt.Str()[0]) {
        OFStream f(rt.Str(), 0);
        f.noansi = 1;
        ShowNews(f, 0, msg, 0);
    }
}

// 5b6a:0bc4: read a key until it is one of valid; Enter gives valid[0].
char GetChoice(const char *valid, int16 upper)
{
    char def = 0;
    if (valid[0] == 0)
        valid++;
    else
        def = valid[0];
    for (;;) {
        char c;
        kin >> c;
        if (c == '\r')
            c = def;
        if (upper)
            c = (char)b_toupper((int)(int8)c);
        if (c && strchr(valid, c))
            return c;
    }
}
