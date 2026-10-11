// Menus of the SRCOMM library (segment 1922).
//
// A menu has a name (used to look up "Menu.<name>.*" resources), a title,
// ten colour bytes (high nibble background, low nibble foreground) and up
// to 128 items.  Items may carry up to five extra columns (strings or
// numbers), used for tables like the rankings.
#ifndef SRE_MENU_H
#define SRE_MENU_H

#include "sstring.h"
#include "stream.h"

// A callback object (an object whose first virtual function is called).
struct MenuHook {
    virtual int16 Call() = 0;
    virtual ~MenuHook() {}
};

struct MenuItem {
    String menuname;    // +00
    String text;        // +08
    char key;           // +10
    int16 id;           // +11
    uint8 colors[8];    // +13 bracket, key, text, column 1..5
    int16 ncols;        // +1b
    char coltype[5];    // +1d 's' or 'n'
    const char *colstr[5];  // +22 (for 's')
    int32 colnum[5];        // +22 (for 'n')

    MenuItem(const String &menuname, const String &text, char key, int16 id);
    void LoadColors();                                  // 1922:16b2
    int16 AddColumn(const char *s);                     // 1922:160a
    int16 AddColumn(int32 v);                           // 1922:165e
    void Draw(int16 last, int16 width, int16 *colw);    // 1922:18e3
    int16 TotalWidth(int16 width, int16 *colw);         // 1922:1bea
    int16 ColWidth(int16 col);                          // 1922:1c42
};

class Menu {
public:
    String name;        // +00
    String title;       // +08
    int16 count;        // +10
    int16 columns;      // +12
    MenuHook *prompt;   // +14
    MenuHook *display;  // +18
    uint8 colors[10];   // +1c title, frame, then the items' (bracket, key, text, columns 1..5)
    MenuItem **items;   // +26

    Menu(const String &name, const String &title, uint8 ctitle, uint8 cframe, uint8 cbracket,
         uint8 ckey, uint8 ctext, uint8 ccol1, uint8 ccol2, uint8 ccol3, uint8 ccol4,
         uint8 ccol5);                                      // 1922:001f
    ~Menu();                                                // 1922:03d0
    int16 Add(String text, char key, int16 id);             // 1922:04dc
    MenuItem *Find(int16 id);                               // 1922:085f
    int16 Run(int16 show, int16 def);                       // 1922:0966
    void Show();                                            // 1922:0eca
    int16 Prompt();                                         // 1922:1467
};

// 14f6:03ef: write a String with the translation hook disabled.
OStream &PutUntranslated(OStream &o, const String &s);
// Manipulator form: o << Untr(name).
struct Untr {
    const char *s;
    explicit Untr(const char *p) : s(p) {}
};
inline OStream &operator<<(OStream &o, const Untr &u) { return PutUntranslated(o, String(u.s)); }
// Colour byte (bg << 4 | fg) as a manipulator.
inline Color ByteColor(uint8 c) { return Color(c % 16, c / 16); }

extern char g_itemopen, g_itemclose;     // DS:61e0 '<' '>'
extern char g_titleopen, g_titleclose;   // DS:61e2 '[' ']'
extern char g_defopen, g_defclose;       // DS:61e4 '[' ']'

#endif
