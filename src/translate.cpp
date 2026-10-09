// Output translation (segment 155b).
#include "translate.h"
#include "devices.h"
#include "files.h"
#include "menu.h"
#include "resource.h"
#include <string.h>

Color c_number(11);         // 677e
Color c_bracket(9);         // 6786
Color c_bold(11);           // 678a
Color c_empire(15);         // 678e

struct TransEntry {
    String from;
    String to;
    TransEntry *next;
};

static TransEntry *g_table[0x100];     // DS:c96a
static String g_tr;                    // DS:c962

// 155b:00ce
static void Apply(String &s)
{
    for (int16 pos = 0; pos < s.length(); pos++) {
        uint8 ch = (uint8)s.at(pos);
        for (TransEntry *e = g_table[ch]; e; e = e->next) {
            String from(e->from);
            int16 n = from.length();
            if (strncmp(s.c_str() + pos, e->from.c_str(), (size_t)(uint16)n) == 0) {
                s.erase(pos, n);
                s.insert(pos, e->to);
                pos--;
                pos += e->to.length();
                break;
            }
        }
    }
    String at("@A");
    for (;;) {
        int16 i = s.find(at, 0);
        if (i < 0)
            break;
        s.erase(i, at.length());
        s.insert(i, String("Amit Patel"));
    }
}

// 155b:027c
const char *Translate(const char *s)
{
    if (s[0] == '\\') {
        g_tr = s;
        g_tr.erase(0, 1);
        return g_tr.c_str();
    }
    if (s[0] == 0 || s[1] == 0 || s[2] == 0) {
        g_tr = s;
        return s;
    }
    g_tr = s;
    Apply(g_tr);
    return g_tr.c_str();
}

// 155b:00b2
int16 DisplayLen(const String &s)
{
    Translate(s.c_str());
    return g_tr.length();
}

// 155b:0313
void LoadTranslation()
{
    Resource tf("Output.TranslationFile", (int32)0);
    if (tf.Str()[0]) {
        IFStream f(tf.Str(), 0);
        TransEntry **tail[0x100];
        for (int16 i = 0; i < 0x100; i++) {
            g_table[i] = 0;
            tail[i] = &g_table[i];
        }
        String line;
        int16 lineno = 0;
        while (!f.fail()) {
            line = "";
            for (;;) {
                char c;
                f >> c;
                if (f.fail())
                    break;
                if (c == '\n')
                    break;
                line += c;
            }
            if (line.at(0) == ';' || line.length() == 0)
                continue;
            lineno++;
            Node *l = ParseList(line);
            if (NodeLength(l) != 2) {
                klocal << "Error in translation file:  line " << lineno
                       << " does not have two items on it.\n";
                FreeNode(l);
                continue;
            }
            Node *a = NodeNth(l, 0);
            Node *b = NodeNth(l, 1);
            a = (a && !a->isCons) ? a : 0;
            b = (b && !b->isCons) ? b : 0;
            if (!a || !b) {
                klocal << "Error in translation file:  line " << lineno
                       << " needs to have two strings.\n";
                FreeNode(l);
                continue;
            }
            uint8 ch = (uint8)a->str.at(0);
            TransEntry *e = new TransEntry;
            e->from = a->str;
            e->to = b->str;
            e->next = 0;
            *tail[ch] = e;
            tail[ch] = &e->next;
            FreeNode(l);
        }
        SetTranslateHook(Translate);
    }
    Resource lb("Output.LeftBracket", "[");
    Resource rb("Output.RightBracket", "]");
    const char *l = lb.Str();
    const char *r = rb.Str();
    if (l[0])
        g_itemopen = l[0];
    if (r[0])
        g_itemclose = r[0];
    Resource bc("Output.Bracket.Color", (int32)c_bracket.fg);
    Resource bo("Output.Bold.Color", (int32)c_bold.fg);
    Resource nc("Output.Number.Color", (int32)c_number.fg);
    Resource ec("Output.Empire.Color", (int32)c_empire.fg);
    c_bracket.fg = (int16)bc.Long();
    c_bold.fg = (int16)bo.Long();
    c_number.fg = (int16)nc.Long();
    c_empire.fg = (int16)ec.Long();
}
