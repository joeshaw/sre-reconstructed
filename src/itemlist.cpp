// "5 soldiers, 2 fighters, and 1 general" (segment 15dd).
#include "sre.h"
#include "itemlist.h"

// 15dd:000c
ItemList::ItemList(Color t, Color n) : skipzero(0), count(0), text(t), number(n)
{
    for (int16 i = 0; i < 0x20; i++)
        items[i].n = 0;
}

// 15dd:00e8: plurals add an 's'.
ItemList &ItemList::Add(int32 n, String name)
{
    if (count >= 0x20)
        return *this;
    if (skipzero && n == 0)
        return *this;
    items[count].n = n;
    items[count].name = name;
    if (n != 1)
        items[count].name += 's';
    count++;
    return *this;
}

// 15dd:01e8: plurals add the given suffix.
ItemList &ItemList::Add(int32 n, String name, String plural)
{
    if (count >= 0x20)
        return *this;
    if (skipzero && n == 0)
        return *this;
    items[count].n = n;
    items[count].name = name;
    if (n != 1)
        items[count].name = items[count].name + plural;
    count++;
    return *this;
}

// 15dd:0368
OStream &operator<<(OStream &o, const ItemList &l)
{
    if (l.count == 0)
        return o << l.text << "nothing";
    for (int16 i = 0; i < l.count; i++) {
        if (l.count - 1 == i && l.count > 1)
            o << l.text << "and ";
        o << l.number << l.items[i].n << ' ' << l.text << l.items[i].name;
        if (l.count - 1 > i) {
            if (l.count > 2)
                o << ',';
            o << ' ';
        }
    }
    return o;
}
