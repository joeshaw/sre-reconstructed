// Research developments: Text/Research.LST compiled into
// Data/Research.DAT (segment 6db3).
#include "sre.h"
#include "research.h"
#include <sys/stat.h>

static const char *RESEARCH_DAT = "Data/Research.DAT";     // DS:a514
static const char *RESEARCH_LST = "Text/Research.LST";     // DS:a518
static int16 g_researchcount = -1;                         // DS:a51c

static int32 MTime(const char *name)
{
    struct stat st;
    if (stat(HostPath(name).c_str(), &st) != 0)
        return 0;
    return (int32)st.st_mtime;
}

// 6db3:063c
void ResearchLoad(Development *d, int16 index)
{
    memset((uint8 *)d + 0xe, 0, 0xf1);
    DataFile f(RESEARCH_DAT, 0xff, 0, 1);
    if (index < 0 || !f.read(index, d, 1)) {
        d->index = index;
        d->cost = 0;
        d->type = 0;
        d->kind = 0;
        d->amount = 0;
    }
}

// 6db3:0000: recompile the list if it is newer than the data file.
int16 ResearchInit()
{
    int32 lst = MTime(RESEARCH_LST);
    int32 dat = MTime(RESEARCH_DAT);
    if ((uint32)lst <= (uint32)dat)
        return 0;
    klocal << Color(3) << "Recompiling " << RESEARCH_LST << " to " << RESEARCH_DAT << "..."
           << c_normal;
    IFStream in(RESEARCH_LST, 0);
    HostRemove(RESEARCH_DAT);
    DataFile out(RESEARCH_DAT, 0xff, 0, 1);
    int32 n = 0, line = 0;
    while (!in.fail()) {
        String s;
        ReadLine(in, s);
        line++;
        Node *list = ParseList(s);
        if (NodeLength(list) == 0)
            continue;
        const char *err = 0;
        Node *items[5] = { 0, 0, 0, 0, 0 };
        if (NodeLength(list) != 5) {
            err = ":  5 items needed.\n";
        } else {
            for (int16 i = 0; i < 5; i++)
                items[i] = NodeNth(list, i);
            if (!items[0] || items[0]->type != AT_SYMBOL)
                err = ":  item 1 should be a name.\n";
            else if (!items[1] || items[1]->type != AT_SYMBOL)
                err = ":  item 2 should be a name.\n";
            else if (!items[2] || items[2]->type != AT_NUMBER)
                err = ":  item 3 should be a number.\n";
            else if (!items[3] || items[3]->type != AT_NUMBER)
                err = ":  item 4 should be a number.\n";
            else if (!items[4] || items[4]->type != AT_STRING)
                err = ":  item 5 should be a string.\n";
        }
        if (err) {
            klocal << "Error on line " << line << err;
            continue;
        }
        String planet(items[0]->str), duration(items[1]->str), text(items[4]->str);
        int16 kind = 0;
        if (StrEqI("permanent", duration))
            kind = 1;
        if (StrEqI("temporary", duration))
            kind = 2;
        if (StrEqI("menu", duration))
            kind = 3;
        if (kind == 0) {
            klocal << "Error on line " << line
                   << ":  duration must be 'temporary' or 'permanent'.\n";
            continue;
        }
        int16 t;
        for (t = 0; t < 10; t++) {
            if (!StrEqI(PlanetName((int8)t), planet))
                continue;
            Development d;
            ResearchLoad(&d, -1);
            d.index = (int16)n;
            d.type = t;
            d.cost = items[3]->num;
            d.amount = items[2]->num;
            d.kind = kind == 1 ? 1 : kind == 2 ? 2 : 0;
            strncpy(d.text, text.c_str(), 0xef);
            d.text[0xef] = 0;
            out.write(n++, &d, 1);
            break;
        }
        if (t == 10)
            klocal << "Error on line " << line << ":  planet type not found.\n";
    }
    kout << "\n\n";
    return (int16)n;
}

// 6db3:05da
int16 ResearchCount()
{
    if (g_researchcount != -1)
        return g_researchcount;
    DataFile f(RESEARCH_DAT, 0xff, 0, 1);
    g_researchcount = (int16)f.numrecs();
    return g_researchcount;
}

// 6db3:073b
int32 Development::Permanent() const
{
    return kind == 1 ? amount : 0;
}

// 6db3:075d
int32 Development::Temporary() const
{
    return kind == 2 ? amount : 0;
}

static int16 Hex(char c)
{
    return c > '@' ? (int16)(c - 55) : (int16)(c - 48);
}

// 6db3:077f: the text, with "^\BF" color codes.
void Development::Show() const
{
    int16 len = (int16)strlen(text);
    for (int16 i = 0; i < len; i++) {
        if (text[i] == '^' && text[i + 1] == '\\') {
            int16 bg = Hex((char)b_toupper((int)(int8)text[i + 2]));
            int16 fg = Hex(text[i + 3]);
            if (bg >= 0 && bg <= 15 && fg >= 0 && fg <= 15) {
                i += 3;
                if (bg == 0)
                    kout << Color(fg, -1);
                else
                    kout << Color(fg, bg);
            } else {
                kout << text[i];
            }
        } else {
            kout << text[i];
        }
    }
    kout << '\n';
}
