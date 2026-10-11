// Pirates (segment 4b60).
#include "sre.h"

// 4b60:0000 (constructor) + 4b60:0090
void PirateNew(Pirate *p)
{
    memset(p->name, 0, 0x20);
    p->level = 0;
    PirateInit(p);
}

// 4b60:0090
void PirateInit(Pirate *p)
{
    p->lasthour = g_now;
    p->money = 0;
    p->food = 0;
    p->soldiers = 0;
    p->fighters = 0;
    p->stations = 0;
    p->heavycruisers = 0;
    p->carriers = 0;
    p->generals = 0;
    p->agents = 0;
    p->color = 7;
    for (int16 i = 0; i < 16; i++)
        p->planets[i] = 0;
    for (int16 i = 0; i < 25; i++)
        p->hatred[i] = 0;
    for (int16 i = 0; i < 0x69; i++)
        p->u_026[i] = 0;
}

// 4b60:01ab
void PirateSetName(Pirate *p, const char *s)
{
    int16 i;
    for (i = 0; s[i] && i < 0x1f; i++)
        p->name[i] = s[i];
    p->name[i] = 0;
}

// 4b60:01ed
int32 PiratePlanets(Pirate *p)
{
    int32 n = 0;
    for (int16 i = 0; i < 16; i++)
        n += p->planets[i];
    return n;
}

// 4b60:022f
int32 PirateNetWorth(Pirate *p)
{
    return 10 * PiratePlanets(p) + p->money / 100000 + 7 * p->agents + p->soldiers + p->fighters + p->food / 10000 +
           3 * p->heavycruisers + (p->stations << 1) + (p->carriers << 1) + 5 * p->generals;
}

// 4b60:034a: pirates rebuild once an hour.
void PirateHourly(Pirate *p)
{
    int32 now = b_time(0);
    if (now - p->lasthour < 3600)
        return;
    p->lasthour = now;
    int16 lv = p->level;
    if ((int32)(int16)(lv * 3000) > p->soldiers)
        p->soldiers += (int32)Random(100) + (5 * (int32)p->planets[8]) / 10;
    if ((int32)(int16)(lv * 3000) > p->fighters)
        p->fighters += (int32)Random(100) + (4 * (int32)p->planets[8]) / 10;
    if ((int32)(int16)(lv * 3000) > p->heavycruisers)
        p->heavycruisers += (int32)Random(100) + (int32)p->planets[8] / 10;
    if ((int32)(int16)(lv * 3000) > p->stations)
        p->stations += (int32)Random(100) + (4 * (int32)p->planets[8]) / 10;
    if ((int32)(int16)(lv * 300) > p->carriers)
        p->carriers += (int32)Random(10) + (2 * (int32)p->planets[8]) / 10;
    if ((int32)(int16)(lv * 300) > p->generals)
        p->generals += (int32)Random(10) + (2 * (int32)p->planets[8]) / 10;
    if ((int32)(int16)(lv * 500) > p->agents)
        p->agents += (int32)Random(10);
    if (10000000 * (int32)lv > p->food)
        p->food += (int32)Random(30) + 6 * (int32)p->planets[3];
    if (10000000 * (int32)lv > p->money) {
        p->money += 10000 * (int32)lv;
        p->money += 230 * (int32)p->planets[0];
        p->money += 50 * (int32)p->planets[1];
        p->money += 260 * (int32)p->planets[2];
        p->money += 210 * (int32)p->planets[5];
        p->money += 70 * (int32)p->planets[7];
    }
    if (10000 * (int32)lv > p->agents)
        p->agents += (int32)p->planets[6] << 1;
}

static const char *const ADJ[19] = {
    "Arcadian", "Galactic", "Extraterrestrial", "Intergalactic", "Infinite", "SRE", "Solar",
    "Space", "Storm", "Xandorian", "Freedom", "Planetary", "Doomed", "Compiler", "Nemesis",
    "Uncertain", "Empire", "Gooey", "Runtime" };
static const char *const NOUN[17] = {
    "Gangs", "Invaders", "Looters", "Marauders", "Raiders", "Twins", "Thieves", "Thugs", "Babies",
    "Pirates", "Dieters", "Bugs", "Blobs", "Nimwits", "Warhogs", "Sharks", "Rogues" };
static const int16 COLORS[9] = { 10, 15, 3, 11, 9, 13, 12, 14, 6 };

// 4b60:088b: name the pirate gangs if they have not been set up.
void CreatePirates()
{
    Pirate *p0 = &g_pirates[0];
    int16 need = !(p0->level > 0 && p0->name[0] && p0->color > 0 && p0->color <= 15);
    b_srand((uint16)b_time(0));
    if (!need)
        return;
    int16 na = 19, nb = 17;
    int16 a = Random(na);
    int16 b = Random(nb);
    int16 sa, sb;
    do
        sa = Random(na - 1) + 1;
    while (na % sa == 0);
    do
        sb = Random(nb - 1) + 1;
    while (nb % sb == 0);
    for (int16 i = 1; i <= 9; i++) {
        Pirate *p = &g_pirates[i - 1];
        PirateInit(p);
        p->level = i;
        char key[16] = "Pirate0.Name";
        char ckey[16] = "Pirate0.Color";
        key[6] = (char)('0' + i);
        ckey[6] = (char)('0' + i);
        char name[0x28];
        Resource rn(key, "");
        if (strlen(rn.Str()) > 0) {
            strcpy(name, rn.Str());
        } else {
            strcpy(name, ADJ[a]);
            strcat(name, " ");
            strcat(name, NOUN[b]);
        }
        PirateSetName(p, name);
        p->color = (uint8)COLORS[i - 1];
        Resource rc(ckey, (int32)0);
        if (rc.Long() > 0)
            p->color = (uint8)rc.Long();
        a = (int16)((a + sa) % na);
        b = (int16)((b + sb) % nb);
    }
}

// 4b60:0bdf
int16 PirateCount()
{
    int16 n = 0;
    for (int16 i = 1; i <= 9; i++) {
        Pirate *p = &g_pirates[i - 1];
        n += (p->level > 0 && p->name[0]) ? 1 : 0;
    }
    return n;
}

// 4b60:0c38
const char *PirateName(Pirate *p)
{
    static char buf[0x80];      // DS:c936
    char key[16] = "Pirate0.Name";
    key[6] = (char)(p->level + '0');
    Resource r(key, "");
    if (strlen(r.Str()) > 0) {
        strcpy(buf, r.Str());
        return buf;
    }
    return p->name;
}

// 4b60:0ced
uint8 PirateColor(Pirate *p)
{
    char key[16] = "Pirate0.Color";
    key[6] = (char)(p->level + '0');
    Resource r(key, (int32)0);
    if (r.Long() > 0)
        return (uint8)r.Long();
    return p->color;
}

// 4b60:0d87
OStream &PutPirate(OStream &o, Pirate *p)
{
    o << Color(PirateColor(p));
    PutUntranslated(o, String(PirateName(p)));
    return o << c_normal;
}
