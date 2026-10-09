// Borland conio on the sysop's local screen (the C library's text-mode
// functions, and the window streams of segment 1fd3 built on them).
//
// The host terminal stands in for the PC's screen: positioning, clearing
// and colours become ANSI sequences, and the terminal's cursor and
// editing keys become the PC's extended key codes.  The local screen is
// taken to be 80x25, all of it the initial window (the original kept
// its top and bottom lines for the sysop's status bars).
#ifndef SRE_CONIO_H
#define SRE_CONIO_H

#include "stream.h"

struct TextInfo {
    uint8 winleft, wintop, winright, winbottom;
    uint8 attribute;
    uint8 screenwidth, screenheight;
};

void gettextinfo(TextInfo *t);                  // 1000:34ac
void window(int16 left, int16 top, int16 right, int16 bottom);  // 1000:352f
void gotoxy(int16 x, int16 y);                  // 1000:2573
void clrscr();                                  // 1000:1afc
void clreol();                                  // 1000:1acd
void insline();                                 // 1000:2668
void delline();                                 // 1000:2686
void textattr(int16 attr);                      // 1000:1b53
void cputs(const char *s, int16 n);             // cprintf("%s")
void setcursortype(int16 type);                 // 1000:1e6a (0 none, 2 normal)
int16 kbhit();                                  // 1000:2825
int16 bioskey0();                               // 1000:1aa8(0): scan << 8 | ascii

// 1fd3:0471: set the window and remember it.
void SetWindow(int16 left, int16 top, int16 right, int16 bottom);
// 1fd3:04a2: the whole screen as the window.
void FullWindow();

// The local screen's window stream (vtable bb65, DS:d2f6): colours take
// effect at once.
class WindowOut : public OStream {
public:
    WindowOut() : OStream(0) {}
    void SetColor(int16 bg, int16 fg);          // 1fd3:0342
    void EmitColor() {}                         // 1fd3:0181
    void RawWrite(const char *s, int16 n);      // 1fd3:00f6
    WindowOut &At(int16 x, int16 y);            // 1fd3:03ab
    void Clear();                               // 1fd3:05f1
    void ClearEol();                            // 1fd3:0630
};

// Keys from the local keyboard (vtable bb49, DS:d316).  An extended key
// reads as 0 and then its scan code.
class WindowIn : public IStream {
public:
    int16 pending;      // +10
    WindowIn() : pending(0) {}
    void Echo(char c);                          // 1fd3:0322
    int16 KeyReady();                           // 1fd3:0217
    char GetKey();                              // 1fd3:0268
};

extern WindowOut kwin;          // DS:d2f6
extern WindowIn kwinin;         // DS:d316

// 1fd3:090d
char WindowGetChar();

// Scrolling pick lists (segments 2858 and 28b5): a column of strings in
// the current window, chosen with the cursor keys.
class PickList {
public:
    int16 count;        // +02
    int16 current;      // +04
    int16 capacity;     // +06
    int16 normal;       // +08 attribute of the items
    int16 highlight;    // +0a attribute of the current item
    String **items;     // +10
    int16 top;          // +14 (ScrollList) first item shown
    int16 height;       // +16
    int16 width;        // +18
    PickList(int16 normal, int16 highlight);    // 2858:000f, 28b5:000e
    ~PickList();                                // 2858:00d4
    void Add(const char *s);                    // 2858:0189
    const char *Item(int16 n);                  // 2858:02cb (1-based)
    int16 Run();                                // 2858:03dd: 1-based choice, 0 for none
private:
    void DrawAll();                             // 28b5:00ee
    void Show(int16 index, int16 attr);                 // 28b5:011e
    void DrawItem(int16 index, int16 row, int16 attr);  // 2858:0306
};

#endif
