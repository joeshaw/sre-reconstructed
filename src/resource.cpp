// RESOURCE.DAT configuration system (segments 2932 and 2a0c).
#include "resource.h"
#include "brtl.h"
#include "files.h"
#include <string.h>

int g_argc;
char **g_argv;

Node g_nil, g_true, g_false;

struct ResEntry {
    String name;
    Node *value;
    ResEntry *next;
};
static ResEntry *g_reslist = 0;
static int16 g_resloaded = 0;
static char g_node[4];

Node::Node() : isCons(false), num(0), type(AT_ERROR), car(0), cdr(0), shared(false) {}

static Node *NewAtom()
{
    Node *n = new Node();
    n->str = "";
    return n;
}

void FreeNode(Node *n)
{
    if (!n || n->shared)
        return;
    if (n->isCons) {
        FreeNode(n->car);
        FreeNode(n->cdr);
    }
    delete n;
}

Node *NodeCar(Node *list)
{
    if (list && list->isCons)
        return list->car;
    return &g_nil;
}

Node *NodeCdr(Node *list)
{
    if (list && list->isCons)
        return list->cdr;
    return 0;
}

Node *NodeNth(Node *list, int16 n)
{
    Node *p = list;
    for (; n > 0 && p; n--)
        p = p->cdr;
    if (!p)
        return &g_nil;
    return p->car;
}

int16 NodeLength(Node *list)
{
    int16 n = 0;
    for (; list; list = list->cdr)
        n++;
    return n;
}

static Node *ReadItem(const String &s, int16 &pos);

// 2a0c:019a: read a symbol.
static Node *ReadSymbol(const String &s, int16 &pos)
{
    while (s.at(pos) && (ctype_signed(s.at(pos)) & CT_SP))
        pos++;
    Node *a = NewAtom();
    for (;;) {
        char c = s.at(pos);
        if (c == 0)
            break;
        if (!((int8)c > 0x20))
            break;
        if (!((uint8)c < 0xff))
            break;
        if (c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}' ||
            c == '\'' || c == '"')
            break;
        a->str += c;
        pos++;
    }
    a->type = AT_SYMBOL;
    return a;
}

// 2a0c:0828: read list items until a closing bracket or the end.
static Node *ReadList(const String &s, int16 &pos)
{
    int16 len = s.length();
    Node *head = 0;
    Node **tail = &head;
    while (pos < len) {
        Node *item = ReadItem(s, pos);
        Node *cell = new Node();
        cell->isCons = true;
        cell->car = item;
        cell->cdr = 0;
        if (item == &g_nil) {
            delete cell;
            break;
        }
        *tail = cell;
        tail = &cell->cdr;
    }
    return head;
}

// 2a0c:031f
static Node *ReadItem(const String &s, int16 &pos)
{
    while (s.at(pos) && (ctype_signed(s.at(pos)) & CT_SP))
        pos++;
    char c = s.at(pos);
    if (c == 0)
        return 0;
    if (c == '"') {
        Node *a = NewAtom();
        for (;;) {
            pos++;
            c = s.at(pos);
            if (c == 0 || c == '"')
                break;
            if (c == '~') {
                pos++;
                c = s.at(pos);
                switch (c) {
                case 'n': c = '\n'; break;
                case 'b': c = 8; break;
                case 't': c = 9; break;
                case 'e': c = 0x1b; break;
                case 'a': c = 7; break;
                case 'v': c = 0x0b; break;
                case 'r': c = '\r'; break;
                default: break;
                }
            }
            a->str += c;
        }
        if (s.at(pos) != '"') {
            FreeNode(a);
            return &g_nil;
        }
        pos++;
        a->type = AT_STRING;
        return a;
    }
    if (c == '(' || c == '[' || c == '{') {
        pos++;
        Node *l = ReadList(s, pos);
        return l ? l : 0;
    }
    if (c == ')' || c == ']' || c == '}') {
        pos++;
        return &g_nil;
    }
    if (c == '#') {
        pos++;
        c = s.at(pos);
        if (c == 'f' || c == 'F') {
            pos++;
            return &g_false;
        }
        if (c == 't' || c == 'T') {
            pos++;
            return &g_true;
        }
        return &g_nil;
    }
    if (c == '\'') {
        pos++;
        Node *a = ReadSymbol(s, pos);
        if (a->type == AT_SYMBOL)
            a->type = AT_QSYMBOL;
        return a;
    }
    Node *a = ReadSymbol(s, pos);
    char f = a->str.at(0);
    if ((ctype_signed(f) & CT_DIG) ||
        ((f == '+' || f == '-') && a->str.at(1) != 0)) {
        int16 i;
        for (i = 1; i < a->str.length(); i++)
            if (!(ctype_signed(a->str.at(i)) & CT_DIG))
                return a;
        a->num = b_atol(a->str.c_str());
        a->type = AT_NUMBER;
    }
    return a;
}

// 2a0c:0b1b: parse a whole line into a list.
Node *ParseList(const String &s)
{
    int16 pos = 0;
    int16 len = s.length();
    Node *head = 0;
    Node **tail = &head;
    while (pos < len) {
        Node *item = ReadItem(s, pos);
        if (!item)
            break;
        Node *cell = new Node();
        cell->isCons = true;
        cell->car = item;
        *tail = cell;
        tail = &cell->cdr;
        if (item == &g_nil)
            break;
    }
    return head;
}

static void InitAtoms()
{
    g_nil.shared = g_true.shared = g_false.shared = true;
    g_nil.type = AT_ERROR;
    g_true.type = AT_BOOL;
    g_true.num = 1;
    g_false.type = AT_BOOL;
    g_false.num = 0;
}

// 2932:0912
void LoadResourceFile(const char *path)
{
    IFStream f(path, 0);
    String line;
    if (f.fail())
        return;
    for (;;) {
        ReadLine(f, line);
        if (f.fail())
            break;
        if (StrEqI("#EOF", line))
            break;
        if (line.at(0) == ';' || line.at(0) == '#' || line.at(0) == '%')
            continue;
        Node *tokens = ParseList(line);
        Node *first = NodeNth(tokens, 0);
        Node *rest = NodeCdr(tokens);
        if (first == &g_nil || first == 0 || first->isCons || first->type != AT_SYMBOL) {
            FreeNode(tokens);
            continue;
        }
        ResEntry *e = new ResEntry();
        e->name = first->str;
        e->value = rest;
        e->next = g_reslist;
        g_reslist = e;
    }
}

// 2932:0cba: -nX selects RESOURCE.X
static void ParseNodeOption()
{
    for (int i = 1; i < g_argc; i++) {
        const char *a = g_argv[i];
        if (a[0] != '/' && a[0] != '-')
            continue;
        if ((a[1] == 'n' || a[1] == 'N') && a[2]) {
            g_node[0] = a[2];
            g_node[1] = a[3];
            g_node[2] = a[3] ? a[4] : 0;
            g_node[3] = 0;
        }
    }
}

void LoadResources()
{
    if (g_resloaded)
        return;
    InitAtoms();
    ParseNodeOption();
    g_resloaded = 1;
    if (g_node[0]) {
        char fn[16];
        strcpy(fn, "RESOURCE.");
        strcat(fn, g_node);
        LoadResourceFile(fn);
    }
    LoadResourceFile("RESOURCE.DAT");
    LoadResourceFile("RESOURCE.DEF");
}

int16 HasCommandArg(const char *s)
{
    for (int i = 1; i < g_argc; i++) {
        const char *a = g_argv[i];
        if (a[0] == '/' || a[0] == '-')
            continue;
        if (b_stricmp(a, s) == 0)
            return 1;
    }
    return 0;
}

Resource::Resource(const char *n, const char *def)
{
    name = n;
    value = 0;
    resolved = 0;
    owned = 1;
    SetDefault(def);
}

Resource::Resource(const char *n, int32 def)
{
    char buf[0x40];
    name = n;
    value = 0;
    resolved = 0;
    owned = 1;
    b_ltoa(def, buf, 10);
    SetDefault(buf);
}

Resource::~Resource()
{
    if (owned && value)
        FreeNode(value);
}

void Resource::SetDefault(const char *s)
{
    if (owned && value)
        FreeNode(value);
    value = NewAtom();
    if (s) {
        value->str = s;
        value->num = b_atol(s);
        value->type = AT_STRING;
        if (value->num != 0)
            value->type = AT_NUMBER;
    } else {
        value->str = "";
        value->num = 0;
        value->type = AT_STRING;
    }
    owned = 1;
}

void Resource::SetValue(Node *list)
{
    if (owned && value)
        FreeNode(value);
    value = list;
    owned = 0;
}

void Resource::Resolve()
{
    LoadResources();
    if (resolved)
        return;
    for (int i = 1; i < g_argc; i++) {
        int16 v = -1;
        const char *a = g_argv[i];
        if (a[0] != '/' && a[0] != '-')
            continue;
        char buf[0x100];
        strncpy(buf, a + 1, sizeof(buf) - 1);
        buf[sizeof(buf) - 1] = 0;
        char *p = buf;
        while (*p && *p != '=')
            p++;
        if (*p) {
            *p = 0;
            p++;
        }
        if (b_stricmp(buf, name) == 0) {
            v = 1;
            if (*p == '0' || b_toupper(*p) == 'F' || b_toupper(*p) == 'N')
                v = 0;
        }
        if (b_strnicmp(buf, "NO", 2) == 0 && b_stricmp(buf + 2, name) == 0)
            v = 0;
        if (v != -1) {
            char s[2] = { (char)('0' + v), 0 };
            SetDefault(s);
            resolved = 1;
        }
    }
    if (resolved)
        return;
    for (ResEntry *e = g_reslist; e; e = e->next) {
        if (b_stricmp(name, e->name.c_str()) == 0) {
            SetValue(e->value);
            resolved = 1;
        }
    }
}

int32 Resource::Long()
{
    Resolve();
    if (!value)
        return 0;
    if (!value->isCons)
        return value->num;
    Node *car = NodeCar(value);
    if (car && !car->isCons)
        return car->num;
    return 0;
}

Node *Resource::List()
{
    Resolve();
    if (!value)
        return value;
    if (!value->isCons)
        return value;
    Node *car = NodeCar(value);
    if (NodeCdr(value) == 0)
        return car;
    return value;
}

const char *Resource::Str()
{
    Resolve();
    if (!value)
        return "";
    if (!value->isCons)
        return value->str.c_str();
    Node *car = NodeCar(value);
    if (car && !car->isCons)
        return car->str.c_str();
    return "";
}

int16 Resource::Bool()
{
    Resolve();
    if (!value)
        return 0;
    Node *a = value->isCons ? NodeCar(value) : value;
    if (!a || a->isCons)
        return 0;
    if (a->type == AT_NUMBER && a->num != 0)
        return 1;
    if (a->type == AT_BOOL)
        return a->num != 0;
    if (StrEqI("true", a->str) || StrEqI("t", a->str) || StrEqI("on", a->str) ||
        StrEqI("yes", a->str))
        return 1;
    return 0;
}
