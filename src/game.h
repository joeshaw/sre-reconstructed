// Game data structures.  These are the exact binary layouts of the
// original data files (Borland C++, byte-packed, little-endian), so the
// reconstruction reads and writes the original's files.
#ifndef SRE_GAME_H
#define SRE_GAME_H

#include "types.h"
#include "sstring.h"
#include "stream.h"
#include <stddef.h>

#pragma pack(push, 1)

// 28ec: a money amount, kept within +-2,000,000,000.
struct Money {
    int32 v;
    int32 get() const { return v; }
    void set(int32 x);              // 28ec:003d
    void add(int32 x);              // 28ec:00b3
    void sub(int32 x);              // 28ec:0199
    operator int32() const { return v; }
};
// 28ec:029a: would a + b leave the money range?
int16 MoneyOverflow(int32 a, int32 b);

// A diplomatic relation with another empire (4 bytes of bit fields).
struct Treaty {
    uint8 a;            // bits 0-7
    uint8 b1;           // bits 8-15: type (low 5 bits) ...
    uint16 w2;          // bits 16-31
    int16 type() const { return b1 & 0x1f; }
    void settype(int16 t) { b1 = (uint8)((b1 & 0xe0) | (t & 0x1f)); }
    // bits 13-17
    int16 c() const { return (int16)((((uint16)b1 | ((uint16)w2 << 8)) >> 5) & 0x1f); }
    void setc(int16 v)
    {
        uint16 x = (uint16)(b1 | (w2 << 8));
        x = (uint16)((x & 0xfc1f) | ((v & 0x1f) << 5));
        b1 = (uint8)x;
        w2 = (uint16)((w2 & 0xff00) | (x >> 8));
    }
    // bits 18-24
    int16 d() const { return (int16)((w2 >> 2) & 0x7f); }
    void setd(int16 v) { w2 = (uint16)((w2 & 0xfe03) | ((v & 0x7f) << 2)); }
    // bits 25-31
    int16 e() const { return (int16)((w2 >> 9) & 0x7f); }
    void sete(int16 v) { w2 = (uint16)((w2 & 0x01ff) | ((v & 0x7f) << 9)); }
};

// Planets of one type.
struct PlanetSet {
    int32 count;        // +0
    int32 a;            // +4
    int32 b;            // +8
};

enum PlanetType {
    P_PETROLEUM, P_EDUCATION, P_TOURISM, P_FOOD, P_RESEARCH,
    P_ORE, P_GOVERNMENT, P_URBAN, P_SUPPLY, P_ANTIPOLLUTION
};

struct Empire {
    int16 id;                   // 000
    char name[0x20];            // 002
    int16 player;               // 022
    int32 created;              // 024
    int32 lastplay;             // 028
    PlanetSet planets[10];      // 02c
    uint8 u_0a4[0x48];          // 0a4
    Treaty treaty[26];          // 0ec
    int32 soldiers;             // 154
    int32 fighters;             // 158
    int32 stations;             // 15c defense stations
    int32 heavycruisers;        // 160
    int32 carriers;             // 164
    int32 generals;             // 168
    int32 d16c;                 // 16c
    int8 effectiveness;         // 170
    int8 cmdship;               // 171 command ship % completed
    int16 w172;                 // 172
    int32 score;                // 174
    int32 d178;                 // 178
    Money money;                // 17c
    Money m180;                 // 180
    int32 population;           // 184 millions
    int32 d188;                 // 188
    int32 food;                 // 18c megatons
    int16 insurgency;           // 190 0..7
    int16 w192;                 // 192
    int16 year;                 // 194 < 0: turns of protection left
    int16 turnsused;            // 196 today
    int16 turnsperday;          // 198
    int16 w19a;                 // 19a
    Money bank;                 // 19c savings (< 0: loan)
    Money m1a0;                 // 1a0
    int32 d1a4;                 // 1a4
    int16 w1a8;                 // 1a8
    int8 taxrate;               // 1aa
    int16 w1ab;                 // 1ab
    int8 b1ad;                  // 1ad
    int16 w1ae;                 // 1ae
    int32 d1b0;                 // 1b0
    int32 d1b4;                 // 1b4
    int16 flags;                // 1b8
    int16 phase;                // 1ba turn phase
    Money m1bc;                 // 1bc
    Money m1c0;                 // 1c0
    uint8 b1c4;                 // 1c4
    int8 b1c5;                  // 1c5
    int8 b1c6;                  // 1c6
    uint8 u_1c7[0x60];          // 1c7
    uint16 weapons;             // 227 bits: 1 nuclear, 2 chemical, 4 psionic, 8 mothership
    int16 w229;                 // 229
    int32 agents;               // 22b covert agents
    int32 d22f[7];              // 22f
    uint8 u_24b[0x24];          // 24b
    int16 w26f[9];              // 26f
    uint8 u_281[0x12];          // 281
    int32 d293;                 // 293
    int8 b297[6];               // 297
    int32 d29d;                 // 29d
    int16 w2a1;                 // 2a1
    int8 b2a3;                  // 2a3
    uint8 u_2a4[0x1f];          // 2a4
    int16 bonds;                // 2c3 bonds outstanding
    int16 maxbonds;             // 2c5
    int32 d2c7;                 // 2c7
    int32 d2cb;                 // 2cb
    int16 w2cf;                 // 2cf
    int16 w2d1;                 // 2d1
    int8 b2d3[25];              // 2d3
    int32 d2ec;                 // 2ec
    int32 lightcruisers;        // 2f0
    int32 troubles;             // 2f4
    int32 bonuses;              // 2f8
};

// 14f6:000f: integrity check of a record (also its encryption key).
struct Checksum {
    int32 sum;          // 7420:0130 (16-bit sum, zero-extended)
    int32 crc;          // 7420:009a CRC-32
    uint8 salt[8];      // initial key, random
    uint8 key[8];       // final key after encryption
    void clear();
};

// Galaxy.II (DS:cd6a, 0x3c6 bytes).
struct Galaxy {
    char name[0x40];            // 000 (name[0] == 0: no game)
    int32 created;              // 040
    int32 t44;                  // 044
    int32 t48;                  // 048
    int32 t4c;                  // 04c
    int32 enddate;              // 050
    Checksum empire[25];        // 054
    Checksum pirate[9];         // 2ac
    int16 w384;                 // 384
    int16 w386;                 // 386 OR-ed into every empire's flags
    int32 d388;                 // 388
    int16 w38c;                 // 38c
    int32 d38e;                 // 38e
    int32 d392;                 // 392
    int16 solarmaster;          // 396
    int16 planetmaster;         // 398
    int16 weaponsmaster;        // 39a
    int16 grantdays;            // 39c turns are granted every n days
    int16 grantturns;           // 39e turns granted
    int16 protection;           // 3a0 turns of protection
    int16 w3a2;                 // 3a2
    int16 w3a4;                 // 3a4
    int16 w3a6;                 // 3a6
    int32 d3a8;                 // 3a8
    int32 d3ac;                 // 3ac
    int16 w3b0;                 // 3b0
    int16 w3b2;                 // 3b2
    int16 w3b4;                 // 3b4
    int32 d3b6;                 // 3b6
    int32 t3ba;                 // 3ba
    int16 interest;             // 3be
    int16 w3c0;                 // 3c0
    int32 d3c2;                 // 3c2
};

// System.II (DS:d196, 0x142 bytes, not encrypted).
struct System {
    char sysop[0x40];           // 000
    char bbs[0x40];             // 040
    char regto[0x50];           // 080 registered to
    char bank[0x50];            // 0d0 name of the bank
    int32 t120;                 // 120
    int32 t124;                 // 124
    int16 regstate;             // 128 0..5
    Checksum galaxy;            // 12a
};

// Player.II record (0x72 bytes).
struct Player {
    int16 index;                // 00
    char user[0x20];            // 02 BBS user name
    char name[0x20];            // 22 name in the game
    char b42[0x20];             // 42
    int32 created;              // 62
    int32 lastplay;             // 66
    int16 male;                 // 6a
    uint8 prefs1;               // 6c
    uint8 prefs2;               // 6d
    int16 w6e;                  // 6e
    int16 w70;                  // 70
};

// Pirate.II record (0x106 bytes).
struct Pirate {
    char name[0x20];            // 00
    int16 level;                // 20 1..9
    int32 lasthour;             // 22
    uint8 u_026[0x69];          // 26
    uint8 color;                // 8f
    int32 money;                // 90
    int32 d94;                  // 94
    int32 d98;                  // 98
    int32 d9c;                  // 9c
    int32 da0;                  // a0
    int32 da4;                  // a4
    int32 da8;                  // a8
    int32 dac;                  // ac
    int32 db0;                  // b0
    int16 wb4[16];              // b4
    int16 wd4[25];              // d4
};

// DOORFILE.SR (7389, DS:d360, 0x30 bytes).
struct DoorInfo {
    char name[0x20];            // 00
    int16 ansi;                 // 20
    int16 ibm;                  // 22
    int16 pagelen;              // 24
    int16 port;                 // 26 0 = local
    uint16 baud;                // 28
    uint16 lockbaud;            // 2a
    int16 minutes;              // 2c time limit, -1 = none
    int16 loaded;               // 2e
};

#pragma pack(pop)

#define CHECK_OFFSET(T, f, o) static_assert(offsetof(T, f) == (o), #T "." #f)
static_assert(sizeof(Money) == 4, "Money");
static_assert(sizeof(Treaty) == 4, "Treaty");
static_assert(sizeof(PlanetSet) == 12, "PlanetSet");
CHECK_OFFSET(Empire, treaty, 0xec);
CHECK_OFFSET(Empire, soldiers, 0x154);
CHECK_OFFSET(Empire, money, 0x17c);
CHECK_OFFSET(Empire, taxrate, 0x1aa);
CHECK_OFFSET(Empire, flags, 0x1b8);
CHECK_OFFSET(Empire, u_1c7, 0x1c7);
CHECK_OFFSET(Empire, weapons, 0x227);
CHECK_OFFSET(Empire, agents, 0x22b);
CHECK_OFFSET(Empire, w26f, 0x26f);
CHECK_OFFSET(Empire, d293, 0x293);
CHECK_OFFSET(Empire, b297, 0x297);
CHECK_OFFSET(Empire, w2a1, 0x2a1);
CHECK_OFFSET(Empire, bonds, 0x2c3);
CHECK_OFFSET(Empire, b2d3, 0x2d3);
CHECK_OFFSET(Empire, d2ec, 0x2ec);
CHECK_OFFSET(Empire, bonuses, 0x2f8);
static_assert(sizeof(Empire) == 0x2fc, "Empire");
static_assert(sizeof(Checksum) == 0x18, "Checksum");
CHECK_OFFSET(Galaxy, empire, 0x54);
CHECK_OFFSET(Galaxy, pirate, 0x2ac);
CHECK_OFFSET(Galaxy, solarmaster, 0x396);
CHECK_OFFSET(Galaxy, d3ac, 0x3ac);
CHECK_OFFSET(Galaxy, interest, 0x3be);
static_assert(sizeof(Galaxy) == 0x3c6, "Galaxy");
CHECK_OFFSET(System, t120, 0x120);
static_assert(sizeof(System) == 0x142, "System");
static_assert(sizeof(Player) == 0x72, "Player");
static_assert(sizeof(DoorInfo) == 0x30, "DoorInfo");
static_assert(sizeof(Pirate) == 0x106, "Pirate");

// ---------------------------------------------------------------------
// Globals.

#define MAXEMPIRES 25
#define MAXPIRATES 9

extern Empire *g_empires;       // DS:c92e
extern Pirate *g_pirates;  // DS:c932
extern Galaxy g_galaxy;         // DS:cd6a
extern System g_system;         // DS:d196
extern DoorInfo g_door;         // DS:d360
extern int32 g_now;             // DS:d35c time at start-up
extern int32 g_today;           // DS:bbb9 midnight today
extern Empire *g_current;       // DS:2b54
extern int16 g_open;            // DS:3ee6 galaxy open count

inline bool Exists(const Empire &e) { return e.name[0] != 0; }


#endif
