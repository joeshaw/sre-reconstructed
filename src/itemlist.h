// "5 soldiers, 2 fighters, and 1 general" (segment 15dd).
#ifndef SRE_ITEMLIST_H
#define SRE_ITEMLIST_H

#include "stream.h"
#include "sstring.h"

class ItemList {
public:
    int16 skipzero;     // +0
    int16 count;        // +2
    struct Item {
        String name;
        int32 n;
    } items[0x20];      // +4
    Color text;         // +8
    Color number;       // +c

    ItemList(Color text, Color number);                 // 15dd:000c
    ItemList &SkipZero() { skipzero = 1; return *this; }   // 15dd:00d3
    ItemList &Add(int32 n, String name);                // 15dd:00e8
    ItemList &Add(int32 n, String name, String plural); // 15dd:01e8
};

OStream &operator<<(OStream &o, const ItemList &l);     // 15dd:0368

#endif
