// Menus of the SRCOMM library (segment 1922).
#include "menu.h"
#include "devices.h"
#include "resource.h"
#include "translate.h"
#include <string.h>

char g_itemopen = '<', g_itemclose = '>';
char g_titleopen = '[', g_titleclose = ']';
char g_defopen = '[', g_defclose = ']';

static const char BAR = (char)0xc4;

static Node *AsAtom(Node *n) { return (n && !n->isCons) ? n : 0; }
static Node *AsList(Node *n) { return (n && n->isCons) ? n : 0; }

// 14f6:03ef
OStream &PutUntranslated(OStream &o, const String &s)
{
    TranslateFn saved = SetTranslateHook(0);
    o << s.c_str();
    SetTranslateHook(saved);
    return o;
}

// Titles starting with '\' are shown literally (without the '\').
static void PutTitle(const String &t)
{
    if (t.at(0) == '\\')
        PutUntranslated(kout, t.substr(1, 0xff));
    else
        kout << t.c_str();
}

// 1922:001f
Menu::Menu(const String &nm, const String &ti, uint8 c0, uint8 c1, uint8 c2, uint8 c3, uint8 c4,
           uint8 c5, uint8 c6, uint8 c7, uint8 c8, uint8 c9)
    : name(nm), title(ti)
{
    items = new MenuItem *[0x80];
    count = 0;
    columns = 1;
    for (int16 i = 0; i < 0x80; i++)
        items[i] = 0;
    prompt = 0;
    display = 0;
    {
        String rn = String("Menu.") + name + ".Title";
        Resource r(rn.c_str(), (int32)1);
        Node *a = AsAtom(r.List());
        if (a && a->num == 0)
            title = a->str;
    }
    int16 c[10] = { c0, c1, c2, c3, c4, c5, c6, c7, c8, c9 };
    {
        String rn = String("Menu.") + name + ".Colors";
        Resource r(rn.c_str(), "");
        Node *l = AsList(r.List());
        if (l) {
            for (int16 i = 0; i < 10; i++) {
                Node *it = NodeNth(l, i);
                if (it && AsAtom(it) && AsAtom(it)->type == AT_NUMBER)
                    c[i] = (int16)AsAtom(it)->num;
            }
        }
    }
    for (int16 i = 0; i < 10; i++)
        colors[i] = (uint8)c[i];
}

// 1922:03d0
Menu::~Menu()
{
    for (int16 i = 0; i < 0x80; i++)
        delete items[i];
    delete prompt;
    delete display;
    delete[] items;
}

// 1922:04dc
int16 Menu::Add(String text, char key, int16 id)
{
    if (count > 0x80)
        return 0;
    {
        String rn = String("Menu.") + name + "." + IntStr(id) + ".Name";
        Resource r(rn.c_str(), (int32)1);
        Node *a = AsAtom(r.List());
        if (a && a->num == 0)
            text = a->str;
    }
    MenuItem *it = new MenuItem(name, text, key, id);
    items[count] = it;
    for (int16 i = 0; i < 8; i++)
        it->colors[i] = colors[2 + i];
    it->LoadColors();
    count++;
    return count - 1;
}

// 1922:085f
MenuItem *Menu::Find(int16 id)
{
    for (int16 i = 0; i < count; i++)
        if (items[i]->id == id)
            return items[i];
    return 0;
}

// 1922:0966
int16 Menu::Run(int16 show, int16 def)
{
    char c;
    {
        String rn = String("Menu.") + name + ".Columns";
        Resource r(rn.c_str(), (int32)0);
        Node *a = AsAtom(r.List());
        if (a && a->num > 0)
            columns = (int16)a->num;
    }
    if (show) {
        Show();
    } else {
        kout << '\n' << ByteColor(colors[1]) << "\xc4\xc4" << g_titleopen << ByteColor(colors[0]);
        PutTitle(title);
        kout << ByteColor(colors[1]) << g_titleclose << "\xc4\xc4" << c_normal << '\n';
    }
    if (Prompt() && def != -1) {
        for (int16 i = 0; i < count; i++) {
            if (items[i]->id != def)
                continue;
            if ((int8)items[i]->key <= 0x20)
                continue;
            if (colors[3] != 0)
                kout << c_normal << g_defopen << "Enter=" << ByteColor(colors[3]) << items[i]->key
                     << c_normal << g_defclose << "  " << c_input;
            break;
        }
    }
    for (;;) {
        kin >> c;
        c = (char)b_toupper((int)(int8)c);
        if (c == '?') {
            kout << "Display Menu" << c_normal << '\n';
            Show();
            Prompt();
        } else {
            for (int16 i = 0; i < count; i++) {
                if (items[i]->key != c)
                    continue;
                if (items[i]->text.length() > 0)
                    kout << items[i]->text.c_str();
                else
                    kout << c;
                kout << c_normal << '\n';
                return items[i]->id;
            }
            if (c == 'Q' || c == '0')
                c = 0x1b;
            if (c == '\r') {
                if (def != -1) {
                    MenuItem *it = Find(def);
                    if (it && it->text.length() > 0)
                        kout << it->text.c_str();
                }
                kout << c_normal << '\n';
                return def;
            }
        }
        if (c == 0x1b) {
            kout << "Quit" << c_normal << '\n';
            return -1;
        }
    }
}

// 1922:0eca
void Menu::Show()
{
    int16 maxtot = 0;
    int16 maxw = 0;
    int16 colw[5];
    int16 i, k;
    for (i = 0; i < 5; i++)
        colw[i] = 0;
    for (i = 0; i < count; i++) {
        int16 w = DisplayLen(items[i]->text);
        int16 hascols = 0;
        for (k = 0; k < 5; k++) {
            int16 cw = items[i]->ColWidth(k + 1);
            if (cw > 0)
                hascols = 1;
            if (cw > colw[k])
                colw[k] = cw;
        }
        if (hascols)
            w += 2;
        if (w > maxw)
            maxw = w;
    }
    for (i = 0; i < count; i++) {
        int16 t = items[i]->TotalWidth(maxw, colw);
        if (t > maxtot)
            maxtot = t;
    }
    int16 ncols = (int16)(0x4d / (maxtot + 2));
    if (ncols < 1)
        ncols = 1;
    if (columns < ncols)
        ncols = columns;
    if (count < ncols)
        ncols = count;
    int16 width = DisplayLen(title) + 5;
    if (title.at(0) == '\\')
        width--;
    if ((maxtot + 2) * ncols - 2 > width)
        width = (maxtot + 2) * ncols - 2;
    kout << '\n';
    if (DisplayLen(title) > 0) {
        int16 tl = DisplayLen(title) + 2;
        if (title.at(0) == '\\')
            tl--;
        kout << ByteColor(colors[1]);
        for (k = 0; k < (width - tl) / 2; k++)
            kout << BAR;
        kout << g_titleopen << ByteColor(colors[0]);
        PutTitle(title);
        kout << ByteColor(colors[1]) << g_titleclose;
        for (; k < width - tl; k++)
            kout << BAR;
        kout << '\n';
    } else {
        kout << ByteColor(colors[1]);
        for (k = 0; k < width; k++)
            kout << BAR;
        kout << '\n';
    }
    if (display)
        display->Call();
    if (ncols > 0) {
        int16 rows = (int16)((count + ncols - 1) / ncols);
        for (int16 r = 0; r < rows; r++) {
            if (kin.KeyReady())
                break;
            for (i = r; i < count; i += rows) {
                int16 last = (count > i + rows) ? 0 : 1;
                items[i]->Draw(last, maxw, colw);
                if (!last)
                    kout << "  ";
            }
        }
    }
    if (!kin.KeyReady()) {
        kout << ByteColor(colors[1]);
        for (k = 0; k < width; k++)
            kout << BAR;
        kout << '\n';
    }
    kout << c_normal;
}

// 1922:1467
int16 Menu::Prompt()
{
    int16 r;
    if (prompt)
        r = prompt->Call();
    else
        r = 1;
    if (r)
        kout << c_normal << "Which one?  ";
    kout << Color(15);
    return r;
}

// ---------------------------------------------------------------------
// 1922:14e5
MenuItem::MenuItem(const String &mn, const String &t, char k, int16 i)
    : menuname(mn), text(t), key(k), id(i)
{
    ncols = 0;
    for (int16 j = 0; j < 8; j++)
        colors[j] = 7;
    for (int16 j = 0; j < 5; j++) {
        coltype[j] = 0;
        colstr[j] = 0;
        colnum[j] = 0;
    }
    LoadColors();
}

// 1922:16b2
void MenuItem::LoadColors()
{
    int16 c[8];
    for (int16 i = 0; i < 8; i++)
        c[i] = colors[i];
    String rn = String("Menu.") + menuname + "." + IntStr(id) + ".Colors";
    Resource r(rn.c_str(), "");
    Node *l = AsList(r.List());
    if (l) {
        for (int16 i = 0; i < 8; i++) {
            Node *it = NodeNth(l, i);
            if (it && AsAtom(it) && AsAtom(it)->type == AT_NUMBER)
                c[i] = (int16)AsAtom(it)->num;
        }
    }
    for (int16 i = 0; i < 8; i++)
        colors[i] = (uint8)c[i];
}

// 1922:160a
int16 MenuItem::AddColumn(const char *s)
{
    if (ncols >= 5)
        return 0;
    coltype[ncols] = 's';
    colstr[ncols] = s;
    ncols++;
    return 1;
}

// 1922:165e
int16 MenuItem::AddColumn(int32 v)
{
    if (ncols >= 5)
        return 0;
    coltype[ncols] = 'n';
    colnum[ncols] = v;
    ncols++;
    return 1;
}

// 1922:18e3
void MenuItem::Draw(int16 last, int16 width, int16 *colw)
{
    if (DisplayLen(text) == 0)
        return;
    if (key != 0) {
        kout << ByteColor(colors[0]) << g_itemopen << ByteColor(colors[1]) << key
             << ByteColor(colors[0]) << g_itemclose;
    } else {
        kout << "   ";
    }
    kout << ByteColor(colors[2]) << ' ';
    if (ncols > 0 || !last)
        kout << setw(width + 1);
    kout << text.c_str();
    kout << setw(0);
    for (int16 k = 0; k < ncols; k++) {
        int16 cw = colw[k];
        kout << ByteColor(colors[3 + k]);
        if (coltype[k] == 's') {
            kout << setw(-cw);
            if (colstr[k])
                PutUntranslated(kout, String(colstr[k]));
            else
                kout << " ";
            kout << setw(0);
        } else {
            kout << setw(cw);
            if (colnum[k] < 0)
                kout << "-";
            else
                kout << colnum[k];
            kout << setw(0);
        }
    }
    if (last)
        kout << '\n';
}

// 1922:1bea
int16 MenuItem::TotalWidth(int16 width, int16 *colw)
{
    if (ncols == 0)
        return DisplayLen(text) + 4;
    int16 t = width + 5;
    for (int16 k = 0; k < 5; k++)
        t += colw[k];
    return t;
}

// 1922:1c42
int16 MenuItem::ColWidth(int16 col)
{
    if (col > ncols)
        return 0;
    if (coltype[col - 1] == 's') {
        int16 l = (int16)strlen(colstr[col - 1]);
        if (l > 2)
            return l + 2;
        return l;
    }
    int32 v = colnum[col - 1];
    int16 n = 0;
    while (v > 0) {
        n++;
        v /= 10;
        if (v > 0)
            n++;
        v /= 10;
        if (v > 0)
            n += 2;
        v /= 10;
    }
    return n + 2;
}
