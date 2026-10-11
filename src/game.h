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
// The status of a proposal (or of a break) sent by the other empire.
enum { TS_NONE, TS_PROPOSED, TS_REJECTED, TS_ACCEPTED, TS_BROKEN };

struct Treaty {
    uint8 days;         // bits 0-7: days left, 0 = no limit
    uint8 lo;           // bits 8-15: type (low 5 bits) ...
    uint16 hi;          // bits 16-31
    int16 type() const { return lo & 0x1f; }
    void settype(int16 t) { lo = (uint8)((lo & 0xe0) | (t & 0x1f)); }
    // bits 13-17: TS_*
    int16 status() const { return (int16)((((uint16)lo | ((uint16)hi << 8)) >> 5) & 0x1f); }
    void setstatus(int16 v)
    {
        uint16 x = (uint16)(lo | (hi << 8));
        x = (uint16)((x & 0xfc1f) | ((v & 0x1f) << 5));
        lo = (uint8)x;
        hi = (uint16)((hi & 0xff00) | (x >> 8));
    }
    // bits 18-24: % of the ally's cruisers that defend
    int16 cruiserpct() const { return (int16)((hi >> 2) & 0x7f); }
    void setcruiserpct(int16 v) { hi = (uint16)((hi & 0xfe03) | ((v & 0x7f) << 2)); }
    // bits 25-31: % of the ally's soldiers that defend
    int16 soldierpct() const { return (int16)((hi >> 9) & 0x7f); }
    void setsoldierpct(int16 v) { hi = (uint16)((hi & 0x01ff) | ((v & 0x7f) << 9)); }
};

// Planets of one type.
struct PlanetSet {
    int32 count;        // +0
    int32 target;       // +4 the production per planet moves toward this
    int32 output;       // +8 production per planet
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
    int32 tradegoods;           // 16c value of the goods out with trading fleets
    int8 effectiveness;         // 170
    int8 cmdship;               // 171 command ship % completed
    int16 tickets;              // 172 lottery tickets bought today
    int32 score;                // 174
    int32 points;               // 178 tournament points (Solar/Planet/Weapons Master)
    Money money;                // 17c
    Money m180;                 // 180 (unused)
    int32 population;           // 184 millions
    int32 pollution;            // 188 exatons
    int32 food;                 // 18c megatons
    int16 insurgency;           // 190 0..7
    int16 planetsbought;        // 192 this turn, less those released
    int16 year;                 // 194 < 0: turns of protection left
    int16 turnsused;            // 196 today
    int16 turnsperday;          // 198
    int16 bonusyear;            // 19a year of the next protection bonus
    Money bank;                 // 19c savings (< 0: loan)
    Money m1a0;                 // 1a0 (unused)
    int32 loanpayment;          // 1a4 owed on the loan each turn
    int16 w1a8;                 // 1a8 (unused)
    int8 taxrate;               // 1aa
    int16 w1ab;                 // 1ab (unused)
    int8 draftrate;             // 1ad
    int16 w1ae;                 // 1ae (unused)
    int32 favor;                // 1b0 of the Galactic Coordinator
    int32 efficiency;           // 1b4 profit per planet
    int16 flags;                // 1b8
    int16 phase;                // 1ba turn phase
    Money income;               // 1bc last turn's, in thousands
    Money m1c0;                 // 1c0 (unused)
    uint8 shortfalls;           // 1c4 SF_* this turn
    int8 effchange;             // 1c5 change of effectiveness from the army's upkeep
    int8 effnext;               // 1c6 effectiveness after it; < 0: desertion
    // 1c7: what went short this turn (0x60 bytes, cleared each turn)
    int32 neglected[10];        // 1c7 planets of each type left unmaintained
    uint8 u_1ef[0x34];          // 1ef
    int32 starving;             // 223 people not fed
    uint16 weapons;             // 227 bits: 1 nuclear, 2 chemical, 4 psionic, 8 mothership
    int16 attacks;              // 229 made today
    int32 agents;               // 22b covert agents
    // 22f: the empires each covert operation was done to (masks); see OpMask
    int32 aidmask;              // 22f insurgent aid, this turn
    int32 dissensionmask;       // 233 this turn
    int32 demoralizemask;       // 237 this turn
    int32 bombmask;             // 23b today
    int32 hostagemask;          // 23f today
    int32 bribemask;            // 243 empires an agent of ours is bribed in
    int32 setupmask;            // 247 today
    uint8 u_24b[0x20];          // 24b
    int32 othermask;            // 26b never used
    int16 w26f[9];              // 26f (cleared, never used)
    uint8 u_281[0x12];          // 281
    int32 researchpoints;       // 293
    int8 supplyrates[6];        // 297 % of supply planets making soldiers, fighters,
                                //     stations, cruisers, carriers, generals
    int32 d29d;                 // 29d (unused)
    int16 inactivedays;         // 2a1 days left before an inactive empire is deleted
    char bondname[0x20];        // 2a3 the empire's own bonds
    int16 bonds;                // 2c3 bonds outstanding
    int16 maxbonds;             // 2c5
    int32 bondprice;            // 2c7
    int32 bondpayoff;           // 2cb
    int16 bondhours;            // 2cf
    int16 bondturns;            // 2d1
    int8 intercepts[25];        // 2d3 our messages each empire's spies will intercept
    int32 lastnews;             // 2ec when the news was last read
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
    int32 startdate;            // 044 play starts
    int32 lastregen;            // 048 planets last came back onto the market
    int32 joindeadline;         // 04c no joining from this date
    int32 enddate;              // 050
    Checksum empire[25];        // 054
    Checksum pirate[9];         // 2ac
    int16 inflation;            // 384 prices grow with the empire
    int16 localonly;            // 386 OR-ed into every empire's flags
    int32 marketfood;           // 388 megatons for sale
    int16 regenseconds;         // 38c a planet comes back every n seconds
    int32 planetsperturn;       // 38e that can be bought, -1 = no limit
    int32 marketplanets;        // 392 for sale
    int16 solarmaster;          // 396
    int16 planetmaster;         // 398
    int16 weaponsmaster;        // 39a
    int16 grantdays;            // 39c turns are granted every n days
    int16 grantturns;           // 39e turns granted
    int16 protection;           // 3a0 turns of protection
    int16 w3a2;                 // 3a2 (unused)
    int16 lotterytickets;       // 3a4 sold today
    int16 lotterywinner;        // 3a6 empire, -1 = none
    int32 jackpot;              // 3a8 last won
    int32 coordfund;            // 3ac the Galactic Coordinator's, in thousands
    int16 coordtax;             // 3b0 per 1000 credits of income
    int16 nukelimit;            // 3b2 most planets a nuclear bomb destroys
    int16 nukepercent;          // 3b4 of the planets a nuclear bomb destroys
    int32 oilprice;             // 3b6
    int32 lasthourly;           // 3ba hourly maintenance last ran
    int16 interest;             // 3be
    int16 newspaper;            // 3c0 that prints the news, 0..3
    int32 psionicprice;         // 3c2 the smuggler's price of a psionic bomb
};

// System.II (DS:d196, 0x142 bytes, not encrypted).
struct System {
    char sysop[0x40];           // 000
    char bbs[0x40];             // 040
    char regto[0x50];           // 080 registered to
    char bank[0x50];            // 0d0 name of the bank
    int32 created;              // 120
    int32 inuse;                // 124 since when the game is in use, 0 = not
    int16 regstate;             // 128 0..5
    Checksum galaxy;            // 12a
};

// Player.II record (0x72 bytes).
struct Player {
    int16 index;                // 00
    char user[0x20];            // 02 BBS user name
    char name[0x20];            // 22 name in the game
    char b42[0x20];             // 42 (cleared, never used)
    int32 created;              // 62
    int32 lastplay;             // 66
    int16 male;                 // 6a
    uint8 prefs1;               // 6c PF1_*
    uint8 prefs2;               // 6d PF2_*
    int16 w6e;                  // 6e (cleared, never used)
    int16 seenplanets;          // 70 bits: planet types whose screen was shown
};

// Pirate.II record (0x106 bytes).
struct Pirate {
    char name[0x20];            // 00
    int16 level;                // 20 1..9
    int32 lasthour;             // 22
    uint8 u_026[0x69];          // 26 (cleared, never used)
    uint8 color;                // 8f
    int32 money;                // 90
    int32 food;                 // 94
    int32 soldiers;             // 98
    int32 fighters;             // 9c
    int32 stations;             // a0
    int32 heavycruisers;        // a4
    int32 carriers;             // a8
    int32 generals;             // ac
    int32 agents;               // b0
    int16 planets[16];          // b4 by type (P_*)
    int16 hatred[25];           // d4 of each empire, from its raids
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
CHECK_OFFSET(Empire, neglected, 0x1c7);
CHECK_OFFSET(Empire, starving, 0x223);
CHECK_OFFSET(Empire, weapons, 0x227);
CHECK_OFFSET(Empire, agents, 0x22b);
CHECK_OFFSET(Empire, othermask, 0x26b);
CHECK_OFFSET(Empire, w26f, 0x26f);
CHECK_OFFSET(Empire, researchpoints, 0x293);
CHECK_OFFSET(Empire, supplyrates, 0x297);
CHECK_OFFSET(Empire, inactivedays, 0x2a1);
CHECK_OFFSET(Empire, bonds, 0x2c3);
CHECK_OFFSET(Empire, intercepts, 0x2d3);
CHECK_OFFSET(Empire, lastnews, 0x2ec);
CHECK_OFFSET(Empire, bonuses, 0x2f8);
static_assert(sizeof(Empire) == 0x2fc, "Empire");
static_assert(sizeof(Checksum) == 0x18, "Checksum");
CHECK_OFFSET(Galaxy, empire, 0x54);
CHECK_OFFSET(Galaxy, pirate, 0x2ac);
CHECK_OFFSET(Galaxy, solarmaster, 0x396);
CHECK_OFFSET(Galaxy, coordfund, 0x3ac);
CHECK_OFFSET(Galaxy, interest, 0x3be);
static_assert(sizeof(Galaxy) == 0x3c6, "Galaxy");
CHECK_OFFSET(System, created, 0x120);
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

// Player::prefs1 (bits 0 and 1 are never used)
enum {
    PF1_SHORTMENUS = 0x04,  // don't redisplay the spending and planet menus
    PF1_TERSE = 0x08,       // shorter prompts (never set by the preferences menu)
    PF1_ASKDEPOSIT = 0x10,  // ask about depositing at the end of a turn
    PF1_DEPOSIT = 0x20,     // ... and deposit by default
    PF1_ASKDONATE = 0x40,   // ask about donating at the end of a turn
    PF1_DONATE = 0x80,      // ... and donate by default
};
// Player::prefs2
enum {
    PF2_ASKPAY = 0x01,      // ask about paying the upkeep
    PF2_AUTOPAY = 0x02,     // ... and pay by default
    PF2_BATTLEMSG = 0x04,   // attach a message to a won battle's report
    PF2_REPORTDATES = 0x08, // show the dates of reports
    PF2_ENTERSTAYS = 0x10,  // Enter doesn't leave the spending menu
};
inline int16 HasPref(uint8 prefs, uint8 f) { return (prefs & f) != 0; }
inline void SetPref(uint8 &prefs, uint8 f, int16 on) { prefs = (uint8)((prefs & ~f) | ((on & 1) ? f : 0)); }

// Empire::shortfalls
// (SF_UNUSED is tested but never set.)
enum { SF_NEGLECT = 1, SF_STARVED = 2, SF_UNUSED = 4, SF_ARMY = 8 };

inline bool Exists(const Empire &e) { return e.name[0] != 0; }


#endif
