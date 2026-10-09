// Research developments (segment 6db3).
#ifndef SRE_RESEARCH_H
#define SRE_RESEARCH_H

#include "types.h"

#pragma pack(push, 1)
// A record of Data/Research.DAT (0xff bytes).
struct Development {
    int16 index;        // 00
    int32 cost;         // 02 research points per planet
    int16 type;         // 06 planet type
    int16 kind;         // 08 1 = permanent, 2 = temporary, 0 = menu
    int32 amount;       // 0a
    char text[0xf1];    // 0e
    int32 Permanent() const;    // 6db3:073b
    int32 Temporary() const;    // 6db3:075d
    void Show() const;          // 6db3:077f
};
#pragma pack(pop)
static_assert(sizeof(Development) == 0xff, "Development");

int16 ResearchInit();                           // 6db3:0000
int16 ResearchCount();                          // 6db3:05da
void ResearchLoad(Development *d, int16 index); // 6db3:063c

#endif
