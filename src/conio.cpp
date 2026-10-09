// Borland conio on the sysop's local screen (see conio.h).
#include "sre.h"
#include "conio.h"
#include <stdio.h>
#include <unistd.h>

WindowOut kwin;
WindowIn kwinin;

static TextInfo g_text = { 1, 1, 80, 25, 7, 80, 25 };
static int16 g_curx = 1, g_cury = 1;   // in the window

// Test scripts (SRE_TESTMARK set) get conio output bracketed by DCS ...
// ST, to tell it from the local stream's.
static void Write(const char *s, int n)
{
    static int marked = getenv("SRE_TESTMARK") != 0;
    if (marked)
        TermWrite("\x1bP", 2);
    TermWrite(s, n);
    if (marked)
        TermWrite("\x1b\\", 2);
}

static void Put(const char *s)
{
    Write(s, (int)strlen(s));
}

// Move the terminal's cursor to (x, y) of the window.
static void Place()
{
    char seq[24];
    snprintf(seq, sizeof(seq), "\x1b[%d;%dH", g_text.wintop + g_cury - 1,
             g_text.winleft + g_curx - 1);
    Put(seq);
}

void gettextinfo(TextInfo *t)
{
    *t = g_text;
}

void window(int16 left, int16 top, int16 right, int16 bottom)
{
    if (left < 1 || top < 1 || right > g_text.screenwidth || bottom > g_text.screenheight ||
        left > right || top > bottom)
        return;
    g_text.winleft = (uint8)left;
    g_text.wintop = (uint8)top;
    g_text.winright = (uint8)right;
    g_text.winbottom = (uint8)bottom;
    gotoxy(1, 1);
}

void gotoxy(int16 x, int16 y)
{
    if (x < 1 || y < 1 || x > g_text.winright - g_text.winleft + 1 ||
        y > g_text.winbottom - g_text.wintop + 1)
        return;
    g_curx = x;
    g_cury = y;
    Place();
}

void clreol()
{
    // The windows used all reach the right edge of the screen.
    Put("\x1b[K");
}

void clrscr()
{
    for (int16 y = 1; y <= g_text.winbottom - g_text.wintop + 1; y++) {
        g_curx = 1;
        g_cury = y;
        Place();
        clreol();
    }
    gotoxy(1, 1);
}

static void LineOp(const char *op)
{
    char seq[64];
    snprintf(seq, sizeof(seq), "\x1b[%d;%dr", g_text.wintop, g_text.winbottom);
    Put(seq);
    Place();
    Put(op);
    Put("\x1b[r");
    g_curx = 1;
    Place();
}

void insline()
{
    LineOp("\x1b[L");
}

void delline()
{
    LineOp("\x1b[M");
}

void textattr(int16 attr)
{
    static const char ansi[] = "04261537";
    char seq[24];
    int16 fg = attr & 15, bg = (attr >> 4) & 7;
    snprintf(seq, sizeof(seq), "\x1b[0;%s3%c;4%cm", fg & 8 ? "1;" : "", ansi[fg & 7], ansi[bg]);
    Put(seq);
    g_text.attribute = (uint8)attr;
}

void cputs(const char *s, int16 n)
{
    Write(s, n);
    for (int16 i = 0; i < n; i++) {
        if (s[i] == '\r')
            g_curx = 1;
        else if (s[i] == '\n')
            g_cury++;
        else if (s[i] == 8)
            g_curx = g_curx > 1 ? g_curx - 1 : 1;
        else if (s[i] != 7)
            g_curx++;
    }
}

void setcursortype(int16 type)
{
    Put(type ? "\x1b[?25h" : "\x1b[?25l");
}

// The next byte of local input without taking it: -1 at the end, -2 if
// none has arrived.  NULs are test-script markers (see devices.cpp).
static int Peek()
{
    for (;;) {
        int c = TermPeek();
        if (c == 0 && getenv("SRE_TESTMARK")) {
            TermGetc();
            continue;
        }
        return c;
    }
}

int16 kbhit()
{
    int c = Peek();
    if (c == -1)
        exit(1);        // the end of the host's input
    return c >= 0;
}

// A terminal escape sequence for a cursor or editing key, as its scan code.
static int16 EscapeKey()
{
    int c = TermGetc();
    int n = 0;
    if (c == '[') {
        for (;;) {
            c = TermGetc();
            if (c >= '0' && c <= '9')
                n = n * 10 + (c - '0');
            else if (c != ';')
                break;
        }
    } else {
        c = TermGetc();
    }
    switch (c) {
    case 'A': return 0x48;
    case 'B': return 0x50;
    case 'C': return 0x4d;
    case 'D': return 0x4b;
    case 'H': return 0x47;
    case 'F': return 0x4f;
    case '~':
        switch (n) {
        case 1: case 7: return 0x47;
        case 4: case 8: return 0x4f;
        case 2: return 0x52;
        case 3: return 0x53;
        case 5: return 0x49;
        case 6: return 0x51;
        }
    }
    return 0;
}

int16 bioskey0()
{
    for (;;) {
        TermFlush();
        while (!kbhit())
            usleep(15000);
        int c = TermGetc();
        if (c == 0x1b) {
            int n = Peek();
            if (n == '[' || n == 'O') {
                int16 scan = EscapeKey();
                if (scan)
                    return (int16)(scan << 8);
                continue;
            }
        }
        if (c == 0x7f)
            c = 8;
        return (int16)c;
    }
}

// ---------------------------------------------------------------------
// 1fd3:0471
void SetWindow(int16 left, int16 top, int16 right, int16 bottom)
{
    window(left, top, right, bottom);
}

// 1fd3:04a2
void FullWindow()
{
    SetWindow(1, 1, g_text.screenwidth, g_text.screenheight);
}

// 1fd3:0342
void WindowOut::SetColor(int16 bg, int16 fg)
{
    if (bg == -1)
        bg = curbg;
    textattr((int16)((bg << 4) | fg));
    curfg = newfg = (int8)fg;
    curbg = newbg = (int8)bg;
}

// 1fd3:00f6
void WindowOut::RawWrite(const char *s, int16 n)
{
    cputs(s, n);
}

// 1fd3:03ab
WindowOut &WindowOut::At(int16 x, int16 y)
{
    gotoxy(x, y);
    return *this;
}

// 1fd3:05f1
void WindowOut::Clear()
{
    gotoxy(1, 1);
    clrscr();
}

// 1fd3:0630
void WindowOut::ClearEol()
{
    clreol();
}

// 1fd3:0322
void WindowIn::Echo(char c)
{
    kwin << c;
}

// 1fd3:0217
int16 WindowIn::KeyReady()
{
    return kbhit() ? 1 : 0;
}

// 1fd3:0268
char WindowIn::GetKey()
{
    if (pending) {
        char c = (char)pending;
        pending = 0;
        return c;
    }
    if (!kwin.raw)
        kwin << "\xdb\b";
    pending = 0;                        // 1fd3:0235
    int16 k = bioskey0();
    if (k & 0xff)
        k &= 0xff;
    if (!kwin.raw)
        kwin << " \b";
    if ((k >> 8) == 0)
        return (char)k;
    if (((k >> 8) & 0xff) == 1)
        return (char)(k & 0xff);
    pending = (int16)((k >> 8) & 0xff);
    return 0;
}

// 1fd3:090d
char WindowGetChar()
{
    char c;
    kwinin >> c;
    return c;
}

// ---------------------------------------------------------------------
// 2858:000f, 28b5:000e
PickList::PickList(int16 n, int16 h)
{
    normal = n;
    highlight = h;
    current = 0;
    capacity = 2;
    items = new String *[capacity];
    for (int16 i = 0; i < capacity; i++)
        items[i] = 0;
    count = 0;
    TextInfo t;
    gettextinfo(&t);
    top = 0;
    width = (int16)(t.winright - t.winleft + 1);
    height = (int16)(t.winbottom - t.wintop + 1);
}

// 2858:00d4
PickList::~PickList()
{
    for (int16 i = 0; i < count; i++)
        delete items[i];
    delete[] items;
}

// 2858:0189
void PickList::Add(const char *s)
{
    if (count == capacity) {
        String **old = items;
        items = new String *[capacity * 2];
        for (int16 i = 0; i < capacity; i++)
            items[i] = old[i];
        for (int16 i = capacity; i < capacity * 2; i++)
            items[i] = 0;
        capacity *= 2;
        delete[] old;
    }
    items[count++] = new String(s);
}

// 2858:02cb
const char *PickList::Item(int16 n)
{
    if (n < 1 || capacity < n)
        return 0;
    return items[n - 1] ? items[n - 1]->c_str() : 0;
}

// 2858:0306
void PickList::DrawItem(int16 index, int16 row, int16 attr)
{
    if (index < 0 || capacity <= index)
        return;
    gotoxy(1, (int16)(row + 1));
    textattr(attr);
    if (items[index])
        cputs(items[index]->c_str(), (int16)items[index]->length());
    clreol();
}

// 28b5:00ee
void PickList::DrawAll()
{
    for (int16 i = 0; i < height; i++)
        Show((int16)(i + top), normal);
}

// 28b5:011e: show an item, scrolling to it.
void PickList::Show(int16 index, int16 attr)
{
    int16 row = (int16)(index - top);
    if (row == -1) {
        gotoxy(1, 1);
        insline();
        DrawItem((int16)(index + index - top), 0, normal);
        top--;
    }
    if (height == row) {
        gotoxy(1, 1);
        delline();
        DrawItem(index, (int16)(height - 1), normal);
        top++;
    }
    if (row < -1 || height < row) {
        if (row < -1)
            top = index;
        if (height < row)
            top = (int16)(index - height + 1);
        for (int16 i = 0; i < height; i++)
            DrawItem((int16)(i + top), i, normal);
    }
    DrawItem(index, (int16)(index - top), attr);
    if (top == index && top > 0 && normal != attr) {
        gotoxy((int16)(width - 1), 1);
        textattr(attr);
        cputs("\x18", 1);
        gotoxy((int16)(width - 1), 1);
    }
    if (top + height - 1 == index && top + height < count && normal != attr) {
        gotoxy((int16)(width - 1), height);
        textattr(attr);
        cputs("\x19", 1);
        gotoxy((int16)(width - 1), height);
    }
}

// 2858:03dd
int16 PickList::Run()
{
    if (current < 0)
        current = 0;
    DrawAll();
    if (count == 0)
        return 0;
    setcursortype(0);
    Show(current, highlight);
    for (;;) {
        int16 prev = current;
        int16 key;
        do {
            key = 0;
            if (kbhit()) {
                key = (uint8)WindowGetChar();
                if (key == 0)
                    key = (int16)((uint8)WindowGetChar() << 8);
            }
            if (!key)
                usleep(15000);
        } while (key == 0);
        if (key == '\r')
            break;
        if (key == 0x1b) {
            current = -1;
            break;
        }
        switch (key) {
        case 0x4800: current--; break;
        case 0x4900: current = 0; break;
        case 0x5000: current++; break;
        case 0x5100: current = (int16)(count - 1); break;
        }
        if (current < 0)
            current = 0;
        if (current >= count)
            current = (int16)(count - 1);
        if (current != prev) {
            Show(prev, normal);
            Show(current, highlight);
        }
    }
    setcursortype(2);
    if (current >= 0 && items[current])
        return (int16)(current + 1);
    return 0;
}
