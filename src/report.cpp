// Reports: messages to empires, kept in stream files (segment 69d7).
#include "sre.h"
#include "report.h"

const char *REPORT_FILE = "Data/Report";       // DS:91e3
int16 g_newreports = 1;                         // DS:91d2
static const char *REPORT_CLASS = "Report";     // DS:91ce

static Streamable *NewReport() { return new Report(); }
static struct ReportReg {
    ClassEntry e;
    ReportReg() { RegisterClass(&e, REPORT_CLASS, NewReport); }     // 69d7:0bbd
} s_reportreg;

// 69d7:0000: one of three rotating 0x240-byte buffers.
char *ReportBuffer()
{
    static char bufs[3][0x240];
    static int16 next;
    next = (int16)((next + 1) % 3);
    return bufs[next];
}

// 69d7:022f
Report::Report()
{
    d.created = g_now;
    d.expires = g_now + 0x93a80;
    d.count = 0;
    d.len = 0;
    memset(d.text, 0, sizeof d.text);
    for (int16 i = 0; i < 25; i++)
        d.to[i] = -1;
}

// 69d7:005f
const char *Report::ClassName()
{
    return REPORT_CLASS;
}

// 69d7:007e: expired when nobody is left to read it, or after a week.
int16 Report::Expired()
{
    if (g_open) {
        for (int16 i = 0; i < MAXEMPIRES; i++)
            if (g_empires[i].name[0] == 0)
                OnEmpireDeleted(i);
    }
    if (d.count == 0)
        return 1;
    return g_now > d.expires;
}

// 69d7:01c9
void Report::OnEmpireDeleted(int16 id)
{
    RemoveRecipient(id);
}

// 69d7:018d
int16 Report::HasRecipient(int16 id)
{
    for (int16 i = 0; i < d.count; i++)
        if (d.to[i] == id)
            return 1;
    return 0;
}

// 69d7:0571
int16 Report::AddRecipient(int16 id)
{
    for (int16 i = 0; i < d.count; i++)
        if (d.to[i] == id)
            return id;
    d.to[d.count++] = id;
    return id;
}

// 69d7:01c9 / 05c5 (all occurrences)
int16 Report::RemoveRecipient(int16 id)
{
    for (int16 i = 0; i < d.count; i++) {
        if (d.to[i] == id) {
            d.count--;
            d.to[i] = d.to[d.count];
            i--;
        }
    }
    return d.count;
}

// 69d7:0624
void Report::AddAll()
{
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (Exists(g_empires[i]))
            AddRecipient(i);
}

// 69d7:0362: "──[n]──", with the date if wanted.
void Report::ShowHeader(int16 n, int16 showtime, int16 c1, int16 c2)
{
    kout << Color(c1) << "\xc4\xc4[" << Color(c2) << n << Color(c1) << "]\xc4\xc4";
    if (showtime) {
        kout << "      " << Color(8);
        PutTime(kout, d.created);
    }
    kout << c_normal << '\n';
}

static int16 HexDigit(char c, int16 def)
{
    c = (char)b_toupper((int)(int8)c);
    if (c >= '0' && c <= '9')
        return (int16)(c - '0');
    if (c >= 'A' && c <= 'F')
        return (int16)(c - 'A' + 10);
    return def;
}

// 69d7:0454: the text, with "^\BF" color codes.
void Report::Show()
{
    kout << c_normal;
    for (uint16 i = 0; i < d.len; i++) {
        char c = d.text[i];
        if (c == '^' && d.text[i + 1] == '\\') {
            Color col(7, -1);
            col.bg = HexDigit(d.text[i + 2], col.bg);
            col.fg = HexDigit(d.text[i + 3], col.fg);
            kout << col;
            i += 3;
        } else {
            kout << c;
        }
    }
    kout << '\n';
}

// 69d7:089d
ReportOut::ReportOut(const char *f) : OStream(0)
{
    file = f ? f : REPORT_FILE;
    raw = 1;
}

// 69d7:0920
ReportOut::ReportOut(int16 id, const char *f) : OStream(0)
{
    if (id < 0 || id >= MAXEMPIRES)
        g_errlog << "Warning:  ReportOut received ID " << id << " in constructor.\n";
    file = f ? f : REPORT_FILE;
    rep.AddRecipient(id);
    raw = 1;
}

// 69d7:09eb
ReportOut::~ReportOut()
{
    AppendObject(file, &rep);
    if (file == REPORT_FILE)
        g_newreports = 1;
}

// 69d7:0a7b
char ReportOut::PutRaw(char c)
{
    if (rep.d.len >= 0x1ff)
        return 0;
    rep.d.text[rep.d.len++] = c;
    rep.d.text[rep.d.len] = 0;
    return c;
}

// 69d7:0acd
void ReportOut::EmitColor()
{
    static const char HEX[] = "0123456789ABCDEF";
    PutRaw('^');
    PutRaw('\\');
    PutRaw(HEX[(int)newbg]);
    PutRaw(HEX[(int)newfg]);
    curfg = newfg;
    curbg = newbg;
}

// 69d7:067c: show the reports for an empire and remove it from their
// recipients.
int16 ShowReports(int16 id, const char *file, int16 showtime, int16 c1, int16 c2)
{
    int16 n = 0;
    StreamFile f(file);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        Report *r = (Report *)o;
        if (r->HasRecipient(id)) {
            n++;
            r->ShowHeader(n, showtime, c1, c2);
            r->Show();
            r->RemoveRecipient(id);
        }
        f.Keep(o);
        delete o;
    }
    return n;
}

// 69d7:0777
int16 ShowNewReports(Player *p, Empire *e)
{
    if (!g_newreports)
        return 0;
    g_newreports = 0;
    return ShowReports(e->id, REPORT_FILE, HasPref(p->prefs2, PF2_REPORTDATES), 1, 3);
}

// 69d7:07bb
int16 ReportsEmpireDeleted(int16 id, const char *file)
{
    int16 n = 0;
    StreamFile f(file ? file : REPORT_FILE);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        o->OnEmpireDeleted(id);
        n++;
        f.Keep(o);
        delete o;
    }
    return n;
}

// 69d7:0b6f: empty a stream file.
void ClearStreamFile(const char *file)
{
    StreamFile f(file ? file : REPORT_FILE);
}
