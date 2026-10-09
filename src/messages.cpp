// Messages between empires: the line editor, sending, reading and replying
// (segment 58b6).
#include "sre.h"
#include "streamfile.h"

static const char *MESSAGE_CLASS = "Message";       // DS:6335
static const char *MESSAGE_FILE = "Data/Message";   // DS:663c
Resource r_messages("Messages", (int32)1);           // 62e8
static Resource r_anonymous("Messages.Anonymous", (int32)1);  // 62f4

enum { MAXLINES = 99, LINELEN = 0x50, QUOTE = '\xff' };

#pragma pack(push, 1)
struct MessageData {
    int16 w0;               // 00
    int16 w2;               // 02
    int16 from;             // 04 sending empire, -1 = sysop/non-player
    int32 created;          // 06
    int32 expires;          // 0a
    char name[0x20];        // 0e sender as shown
    char to[0x20];          // 2e recipient letters
    int16 list[25];         // 4e recipients (public: those who read it)
    int16 count;            // 80
    uint8 flags;            // 82 1 anonymous, 2 public
    int16 nlines;           // 83
    char lines[MAXLINES][LINELEN];  // 85
};
#pragma pack(pop)
static_assert(sizeof(MessageData) == 0x1f75, "MessageData");

enum { MF_ANON = 1, MF_PUBLIC = 2 };

// Class "Message" (vtable 676a).
class Message : public Streamable {
public:
    MessageData d;
    Message() { Clear(); }                                  // 58b6:047e
    const char *ClassName() { return MESSAGE_CLASS; }       // 58b6:01a4
    int16 Size() { return (int16)(d.nlines * LINELEN + 0x87); }  // 58b6:01bb
    uint8 *Data() { return (uint8 *)&d; }
    int16 Expired();                                        // 58b6:01d5
    void OnEmpireDeleted(int16 id);                         // 58b6:01ec
    void Clear();                                           // 58b6:0387
    int16 Receives(int16 id);                               // 58b6:0205
    void MarkRead(int16 id);                                // 58b6:029a
    int16 AddressedTo(int16 id);                            // 58b6:04f6
    void Header(int16 n, int16 c1, int16 c2);               // 58b6:054b
    void ShowInfo(int16 c);                                 // 58b6:0868
    void ShowLines(int16 c1, int16 c2, int16 c3, int16 c4); // 58b6:0961
    void LineNumber(int16 c);                               // 58b6:0aa4
    void Edit(char quote, int16 c);                         // 58b6:0b54
    void AddRecipient(int16 id);                            // 58b6:13ce
    void RemoveRecipient(int16 id);                         // 58b6:1425
    void SortRecipients();                                  // 58b6:1487
    void Address(int16 from, int16 to);                     // 58b6:156a
    int16 PrepareReply(int16 me);                           // 58b6:15db
    int16 ChooseRecipients(int16 from);                     // 58b6:181e
    int16 Compose(String sender, char quote);               // 58b6:195c
};

static Streamable *NewMessage() { return new Message(); }
static struct MessageReg {
    ClassEntry e;
    MessageReg() { RegisterClass(&e, MESSAGE_CLASS, NewMessage); }     // 58b6:2a51
} s_messagereg;

static Message *g_message;      // DS:6300 the message being written

// 58b6:0000
static Message *TheMessage()
{
    if (!g_message)
        g_message = new Message();
    return g_message;
}

// 58b6:009f
int16 Message::Expired()
{
    if (!(d.flags & MF_PUBLIC) && d.count == 0)
        return 1;
    return g_now > d.expires;
}

// 58b6:011c
void Message::OnEmpireDeleted(int16 id)
{
    for (int16 i = 0; i < d.count; i++) {
        if (d.list[i] == id) {
            d.count--;
            d.list[i] = d.list[d.count];
            i--;
        }
    }
    if (d.from == id) {
        d.count = 0;
        d.flags &= ~MF_PUBLIC;
    }
}

// 58b6:0387
void Message::Clear()
{
    d.w0 = -1;
    d.w2 = -1;
    d.from = -1;
    d.count = 0;
    d.created = 0;
    d.expires = 0;
    memset(d.name, 0, 0x20);
    memset(d.to, 0, 0x20);
    for (int16 i = 0; i < 25; i++)
        d.list[i] = -1;
    for (int16 i = 0; i < MAXLINES; i++)
        memset(d.lines[i], ' ', LINELEN);
    d.flags &= ~MF_ANON;
    d.flags &= ~MF_PUBLIC;
    d.nlines = 0;
}

// 58b6:0205: for public messages the list holds those who have read it.
int16 Message::Receives(int16 id)
{
    if (d.flags & MF_PUBLIC) {
        for (int16 i = 0; i < d.count; i++)
            if (d.list[i] == id)
                return 0;
        return 1;
    }
    for (int16 i = 0; i < d.count; i++)
        if (d.list[i] == id)
            return 1;
    return 0;
}

// 58b6:029a
void Message::MarkRead(int16 id)
{
    if (d.flags & MF_PUBLIC) {
        for (int16 i = 0; i < d.count; i++)
            if (d.list[i] == id)
                return;
        if (d.count < 25) {
            d.list[d.count++] = id;
        } else {
            d.count = 0;
            d.flags &= ~MF_PUBLIC;
        }
        return;
    }
    for (int16 i = 0; i < d.count; i++) {
        if (d.list[i] == id) {
            d.count--;
            d.list[i] = d.list[d.count];
            i--;
        }
    }
}

// 58b6:04f6: sent to the empire, rather than seen by its spies.
int16 Message::AddressedTo(int16 id)
{
    if (d.flags & MF_PUBLIC)
        return 1;
    for (int16 i = 0; d.to[i]; i++)
        if ((int8)d.to[i] == id + 'A')
            return 1;
    return 0;
}

// 58b6:054b
void Message::Header(int16 n, int16 c1, int16 c2)
{
    kout << Color(c1) << "\xda";
    for (int16 i = 0; i < 66; i++)
        kout << '\xc4';
    if (n < 1000)
        kout << '\xc4';
    if (n < 100)
        kout << '\xc4';
    if (n < 10)
        kout << '\xc4';
    if (n <= 0)
        kout << '\xc4';
    else
        kout << Color(c2) << n;
    kout << Color(c1) << "\xc4\xc4\n";
    kout << Color(c1) << "\xb3  " << c_normal << "From:    ";
    if (d.from != -1)
        kout << c_empire << d.name << '\n';
    else if (d.name[0])
        kout << c_normal << d.name << '\n';
    else
        kout << Color(14) << "SYSOP\n";
    kout << Color(c1) << "\xb3  " << c_normal << "To:      " << c_empire << d.to << '\n';
    if ((d.flags & MF_ANON) || (d.flags & MF_PUBLIC) || d.from == -1) {
        kout << Color(c1) << "\xb3  " << c_normal << "Flags:   " << Color(c1);
        if (d.flags & MF_ANON)
            kout << "(anonymous) ";
        if (d.flags & MF_PUBLIC)
            kout << "(public) ";
        if (d.from == -1)
            kout << (d.name[0] ? "(non-player) " : "(sysop) ");
        kout << '\n';
    }
}

// 58b6:0868: the details only the sysop sees.
void Message::ShowInfo(int16 c)
{
    if (d.from != -1)
        kout << Color(c) << "\xb3  " << c_normal << "Real sender:  " << c_empire
             << (char)(d.from + 'A') << '\n';
    const char *t = TimeStr(d.created);
    kout << Color(c) << "\xb3  " << c_normal << "Date:         " << t << '\n';
}

// 58b6:0961
void Message::ShowLines(int16 c1, int16 c2, int16 c3, int16 c4)
{
    (void)c2;
    for (int16 i = 0; i < d.nlines; i++) {
        kout << Color(c1) << '\xb3';
        const char *s;
        if (d.lines[i][0] == QUOTE) {
            kout << Color(c3) << d.lines[i][1] << '>' << Color(c4);
            s = d.lines[i] + 2;
        } else {
            kout << "  " << c_normal;
            s = d.lines[i];
        }
        while (*s)
            kout << *s++;
        kout << '\n';
    }
}

// 58b6:0aa4
void Message::LineNumber(int16 c)
{
    if (d.nlines == MAXLINES)
        kout << Color(14) << "END";
    else
        kout << Color(c) << (int16)((d.nlines + 1) / 10) << (int16)((d.nlines + 1) % 10) << ' ';
}

// 58b6:0b3a
static int16 Printable(char c)
{
    return (uint8)c >= 0x20 && (uint8)c <= 0xaf;
}

// 58b6:0b54: the line editor.  With a quote character the old text is
// kept as quoted lines.
void Message::Edit(char quote, int16 c)
{
    Color cmd(14);
    int16 col = 0;
    kout << Color(c)
         << "   [---+----|----+----|----+----|----+----|----+----|----+----|----+----]\n";
    if (quote) {
        int16 n = 0;
        for (int16 i = 0; i < d.nlines && n < 20; i++) {
            if (d.lines[i][0] == QUOTE)
                continue;
            char tmp[0x8e];
            strcpy(tmp, d.lines[i]);
            d.lines[n][0] = QUOTE;
            d.lines[n][1] = quote;
            d.lines[n][2] = 0;
            strcat(d.lines[n], tmp);
            n++;
        }
        d.nlines = n;
        ShowLines(3, 0xb, 9, 3);
    } else {
        d.nlines = 0;
    }
    LineNumber(10);
    for (;;) {
        char k;
        kout << Color(15);
        kin >> k;
        if ((k == '\r' || k == '\n') && d.nlines < MAXLINES) {
            kout << '\n';
            d.lines[d.nlines][col] = 0;
            d.nlines++;
            col = 0;
            LineNumber(10);
            continue;
        }
        if (k == 8 && col > 0) {
            kout << "\b \b";
            col--;
            d.lines[d.nlines][col] = 0;
            continue;
        }
        if (k == 8 && d.nlines > 0 && d.lines[d.nlines - 1][0] != QUOTE) {
            d.nlines--;
            col = (int16)strlen(d.lines[d.nlines]);
            kout << "\b\b\b";
            LineNumber(0xc);
            kout << Color(15) << d.lines[d.nlines];
            continue;
        }
        if (k == '/' && col == 0) {
            kout << Color(15) << '/' << Color(7) << '-';
            kout << Color(14) << "Command " << c_bracket << '[' << c_bold;
            kout << 'S' << Color(3) << "=save " << c_bold;
            kout << 'A' << Color(3) << "=abort " << c_bold;
            kout << 'L' << Color(3) << "=list " << c_bold;
            kout << 'C' << Color(3) << "=clear";
            kout << c_bracket << ']' << c_normal << ' ' << Color(15);
            char a;
            kin >> a;
            for (int16 i = 0; i < 0x2a; i++)
                kout << "\b \b";
            a = (char)b_toupper((int)(int8)a);
            if (a == 'S') {
                kout << cmd << "\b\b\bSave message.\n";
                return;
            }
            if (a == 'A') {
                char y;
                kout << cmd << "Are you sure?  ";
                do {
                    kin >> y;
                    y = (char)b_toupper((int)(int8)y);
                } while (y != 'Y' && y != 'N');
                for (int16 i = 0; i < 0xf; i++)
                    kout << "\b \b";
                if (y == 'Y') {
                    kout << cmd << "\b\b\bAborting...\n";
                    d.nlines = 0;
                    return;
                }
            }
            if (a == 'C') {
                kout << cmd << "\b\b\bCleared message; start over.\n";
                d.nlines = 0;
                col = 0;
                LineNumber(10);
            }
            if (a == 'L') {
                kout << cmd << "\b\b\bList:\n" << c_normal;
                ShowLines(3, 0xb, 9, 3);
                LineNumber(10);
            }
            continue;
        }
        if (Printable(k) && col < 0x45 + (ctype_signed((int8)k) & CT_SP) &&
            d.nlines < MAXLINES) {
            kout << k;
            d.lines[d.nlines][col++] = k;
            continue;
        }
        if (!Printable(k) || d.nlines >= 0x62)
            continue;
        // Wrap the last word onto a new line.
        char *line = d.lines[d.nlines];
        int16 brk = (int16)(col - 1);
        while (!(ctype_signed((int8)line[brk]) & CT_SP) && brk > 0x32)
            brk--;
        if (brk > 0x32)
            brk++;
        else
            brk = col;
        for (int16 i = brk; i < col; i++)
            kout << "\b \b";
        kout << '\n';
        int16 m = 0;
        for (int16 i = brk; i < col; i++)
            d.lines[d.nlines + 1][m++] = line[i];
        line[brk] = 0;
        d.nlines++;
        d.lines[d.nlines][m++] = k;
        d.lines[d.nlines][m] = 0;
        LineNumber(10);
        kout << Color(15) << d.lines[d.nlines];
        col = m;
    }
}

// 58b6:13ce
void Message::AddRecipient(int16 id)
{
    for (int16 i = 0; i < d.count; i++)
        if (d.list[i] == id)
            return;
    d.list[d.count++] = id;
}

// 58b6:1425
void Message::RemoveRecipient(int16 id)
{
    for (int16 i = 0; i < d.count; i++) {
        if (d.list[i] == id) {
            d.count--;
            d.list[i] = d.list[d.count];
            return;
        }
    }
}

// 58b6:1487: sort the recipients and spell them out.
void Message::SortRecipients()
{
    for (int16 i = 0; i < d.count - 1; i++) {
        for (int16 j = (int16)(i + 1); j < d.count; j++) {
            if (d.list[i] > d.list[j]) {
                int16 t = d.list[i];
                d.list[i] = d.list[j];
                d.list[j] = t;
            }
        }
    }
    for (int16 i = 0; i < d.count; i++)
        d.to[i] = (char)(d.list[i] + 'A');
    d.to[d.count] = 0;
}

// 58b6:156a
void Message::Address(int16 from, int16 to)
{
    d.flags &= ~MF_ANON;
    d.flags &= ~MF_PUBLIC;
    if (to == -1) {
        d.count = 0;
    } else {
        d.list[0] = to;
        d.count = 1;
    }
    SortRecipients();
    d.from = from;
}

// 58b6:15bd
static void ListEmpires(int32 mask)
{
    ScoreList(&g_galaxy, mask, 0);
}

// 58b6:15db: turn a received message into a reply; returns the empire
// replied to (-1 if anonymous), or -2 if no one was chosen.
int16 Message::PrepareReply(int16 me)
{
    int16 anon = d.flags & MF_ANON;
    int16 to = d.from;
    if (anon)
        to = -1;
    d.flags &= ~MF_ANON;
    int16 all = 0;
    if (d.flags & MF_PUBLIC)
        all = YesNo(0, "Publicly post your reply");
    else if (d.to[1] == 0 && (int8)d.to[0] - 'A' == me)
        all = 0;
    else
        all = YesNo(0, "Send a copy to all original recipients");
    int32 chosen = 0;
    if (!anon)
        MaskSet(&chosen, d.from);
    if (all) {
        if (!(d.flags & MF_PUBLIC)) {
            for (int16 i = 0; d.to[i]; i++)
                MaskSet(&chosen, (int16)((int8)d.to[i] - 'A'));
            if (d.count != 1)
                MaskClear(&chosen, me);
        }
    } else {
        d.flags &= ~MF_PUBLIC;
        d.list[0] = d.from;
    }
    d.count = 0;
    if (!(d.flags & MF_PUBLIC)) {
        char c = (char)ChooseEmpires(EmpireMask(), &chosen, String("*"), ListEmpires,
                                     "Destinations: ");
        if (anon && YesNo(1, "Include the original (anonymous) author"))
            MaskSet(&chosen, d.from);
        if (c == 0)
            return -2;
        if (c == '*') {
            d.flags |= MF_PUBLIC;
        } else {
            for (int16 i = 0; i < 25; i++)
                if (MaskTest(chosen, i))
                    AddRecipient(i);
            SortRecipients();
        }
    }
    d.from = me;
    return to;
}

// 58b6:181e
int16 Message::ChooseRecipients(int16 from)
{
    Clear();
    int32 chosen = 0;
    char c = (char)ChooseEmpires(EmpireMask(), &chosen, String("*"), ListEmpires,
                                 "Choose Empires [* for a public message] > ");
    if (c == 0)
        return 0;
    if (c == '*') {
        kout << c_input << "(Public Message)" << c_normal << '\n';
        d.count = 0;
        d.flags |= MF_PUBLIC;
        strcpy(d.to, "All");
    } else {
        d.count = 0;
        for (int16 i = 0; i < 25; i++)
            if (MaskTest(chosen, i))
                AddRecipient(i);
        SortRecipients();
        if (d.count == 0)
            return 0;
        d.flags &= ~MF_PUBLIC;
    }
    d.from = from;
    return 1;
}

// 58b6:195c: write the text and sign the message.
int16 Message::Compose(String sender, char quote)
{
    kout << c_normal << "You have " << c_number << (int16)MAXLINES << c_normal
         << " lines for the message.  " << c_bracket << '[' << c_bold << "/S" << c_bracket << ']'
         << c_normal << " to save!\n";
    Edit(quote, 3);
    if (d.nlines == 0)
        return 0;
    kout << c_normal;
    if (r_anonymous.Bool() && YesNo(0, "Anonymous")) {
        strcpy(d.name, "<Anonymous>");
        d.flags |= MF_ANON;
    } else {
        strncpy(d.name, sender.c_str(), 0x1f);
        d.name[0x1f] = 0;
        d.flags &= ~MF_ANON;
    }
    d.expires = g_now + 604800;
    d.created = g_now;
    if (d.flags & MF_PUBLIC) {
        int16 days;
        kout << c_normal
             << "How many days would you like to leave this public message up? [1-30] ";
        kin >> defval(7) >> maxval(30) >> days;
        if (days < 1)
            days = 1;
        if (days > 30)
            days = 30;
        d.expires = g_now + 86400 * (int32)days;
        kout << c_normal;
    }
    return 1;
}

// 58b6:1bee: write a message from an empire to one other empire; the
// caller files it.
Streamable *ComposeMessage(Empire *e, int16 to)
{
    Message *m = TheMessage();
    m->Address(e->id, to);
    if (m->Compose(String(e->name), 0))
        return m;
    return 0;
}

// 58b6:1ce9
int16 SendMessage(int16 from, String name)
{
    Message *m = TheMessage();
    m->Address(from, -1);
    if (!m->ChooseRecipients(from))
        return 0;
    if (!m->Compose(name, 0))
        return 0;
    if (from != -1) {
        // Spies planted in the sender's court get copies.
        for (int16 i = 0; i < 25; i++) {
            if ((uint8)g_empires[from].b2d3[i] > 0 && !m->Receives(i)) {
                m->AddRecipient(i);
                g_empires[from].b2d3[i]--;
            }
        }
    }
    AppendObject(MESSAGE_FILE, m);
    return 1;
}

// 58b6:1eb3
int16 ReadMessages(Player *p, Empire *e)
{
    (void)p;
    int16 n = 0;
    StreamFile f(MESSAGE_FILE);
    for (;;) {
        int32 inpos = f.in.tell();
        int32 outpos = f.out.tell();
        Streamable *o = f.Read();
        if (!o)
            break;
        Message *m = (Message *)o;
        if (!m->Receives(e->id)) {
            f.Keep(o);
            delete o;
            continue;
        }
        n++;
        if (!m->AddressedTo(e->id)) {
            kout << Color(5) << "    (" << Color(13) << "reported by a spy" << Color(5) << ")\n";
            m->Header(n, 5, 0xd);
            m->ShowLines(5, 0xd, 0xc, 6);
        } else {
            m->Header(n, 3, 0xb);
            m->ShowLines(3, 0xb, 9, 3);
        }
        if (m->d.from < 0) {
            Pause();
            m->MarkRead(e->id);
            f.Keep(o);
            delete o;
            continue;
        }
        kout << c_bracket << '[' << c_bold << 'R' << c_bracket << ']' << c_normal << "eply, "
             << c_bracket << '[' << c_bold << 'Q' << c_bracket << ']' << c_normal
             << "uoted reply, or " << c_bracket << '[' << c_bold << 'D' << c_bracket << ']'
             << c_normal << "elete?  " << c_input;
        char c = GetChoice("KRDQ", 1);
        if (c == 'R')
            kout << "Reply";
        if (c == 'Q')
            kout << "Quoted Reply";
        if (c == 'D')
            kout << "Delete";
        if (c == 'K')
            kout << "Keep";
        kout << c_normal << '\n';
        if (c != 'K')
            m->MarkRead(e->id);
        f.Keep(o);
        if (c == 'R' || c == 'Q') {
            int16 to = m->PrepareReply(e->id);
            int16 again = 1;
            if (to >= -1) {
                char quote = '-';
                if (to >= 0 && to < 25)
                    quote = (char)(to + 'A');
                if (c != 'Q')
                    quote = 0;
                if (m->Compose(String(e->name), quote)) {
                    f.Keep(o);
                    again = 0;
                }
            }
            if (again) {
                // Show the message again.
                f.in.seek(inpos);
                f.out.seek(outpos);
            }
        }
        delete o;
    }
    return n;
}

// 58b6:2481
void Communications(Player *p, Empire *e)
{
    (void)p;
    String first = String("Do you wish to send ") + "a" + " message";
    String another = String("Do you wish to send ") + "another" + " message";
    if (!r_messages.Bool())
        return;
    const char *q = first.c_str();
    int16 yes;
    do {
        kout << c_normal;
        yes = YesNo(0, q);
        if (yes && SendMessage(e->id, String(e->name)))
            q = another.c_str();
    } while (yes);
}

// 58b6:25f9
int16 MessagesEmpireDeleted(int16 id)
{
    int16 n = 0;
    StreamFile f(MESSAGE_FILE);
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

// 58b6:26ba: the sysop reads all messages.
int16 SysopMessages()
{
    int16 n = 0;
    StreamFile f(MESSAGE_FILE);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        Message *m = (Message *)o;
        n++;
        m->Header(n, 3, 0xb);
        m->ShowInfo(3);
        m->ShowLines(3, 0xb, 9, 3);
        kout << c_bracket << '[' << c_bold << 'D' << c_bracket << ']' << c_normal << "elete or "
             << c_bracket << '[' << c_bold << 'K' << c_bracket << ']' << c_normal << "eep?  "
             << c_input;
        for (;;) {
            char c;
            kin >> c;
            if (c == 'K' || c == 'k' || c == '\r') {
                kout << "Keep";
                f.Keep(o);
                break;
            }
            if (c == 'D' || c == 'd') {
                kout << "Delete";
                break;
            }
        }
        kout << c_normal << '\n';
        delete o;
    }
    return n;
}

// 58b6:2901
int16 ReadPublicMessages()
{
    int16 n = 0;
    StreamFile f(MESSAGE_FILE);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        Message *m = (Message *)o;
        if (m->d.flags & MF_PUBLIC) {
            n++;
            m->Header(n, 3, 0xb);
            m->ShowLines(3, 0xb, 9, 3);
            Pause();
        }
        f.Keep(o);
        delete o;
    }
    return n;
}

// 58b6:2a17
void ClearMessages()
{
    StreamFile f(MESSAGE_FILE);
}
