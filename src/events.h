// Scheduled events: covert operations and bank investments (4c48).
#ifndef SRE_EVENTS_H
#define SRE_EVENTS_H

#include "streamfile.h"
#include "report.h"
#include "game.h"

extern const char *EVENT_FILE;      // DS:303a "Data/Event"
extern const char *COVERT_FILE;     // DS:308e "Data/CovertOp"
extern int32 g_nextevent;           // DS:c95e

#pragma pack(push, 1)
struct EventData {
    int16 empire;       // 02 owner, -1 when canceled
    int32 created;      // 04
    int32 expires;      // 08
    int32 due;          // 0c
};
struct CovertData : EventData {
    int16 target;       // 10
    int16 success;      // 12
};
struct SetupData : CovertData {
    int16 victim;       // 14 the empire made to believe it was attacked
};
struct BankData : EventData {
    Money amount;       // 10
    Money m14;          // 14
    char name[0x4e];    // 18
    int16 year;         // 66 matures in this year of the empire
    int16 issuer;       // 68 empire that issued the bonds, -1 = bank
};
#pragma pack(pop)

// Base class (vtable 381a).
class Event : public Streamable {
public:
    virtual EventData &ev() = 0;
    void Init(int16 id);                        // 4c48:0000
    void SetDue(int16 days, int16 hours, int16 minutes);  // 4c48:011d
    int16 Expired();                            // 4c48:01b2
    virtual int16 Due();                        // [14] 4c48:0218
    virtual Event *Execute() { return 0; }      // [18] 4c48:0112
};

// Covert operations (vtable 37fa).
class EventCovert : public Event {
public:
    virtual CovertData &cv() = 0;
    EventData &ev() { return cv(); }
    void Init(int16 id, int16 target, int16 success);   // 4c48:0392
    void OnEmpireDeleted(int16 id);             // 4c48:046c
    virtual void Display() {}                   // [1c]
};

class EventCovertDissension : public EventCovert {
public:
    CovertData d;
    EventCovertDissension(int16 id = -1, int16 target = -1, int16 success = 0);
    CovertData &cv() { return d; }
    uint8 *Data() { return (uint8 *)&d; }
    const char *ClassName();
    int16 Size() { return 0x14; }
    Event *Execute();
    void Display();
};

class EventCovertDemoralize : public EventCovert {
public:
    CovertData d;
    EventCovertDemoralize(int16 id = -1, int16 target = -1, int16 success = 0);
    CovertData &cv() { return d; }
    uint8 *Data() { return (uint8 *)&d; }
    const char *ClassName();
    int16 Size() { return 0x14; }
    Event *Execute();
    void Display();
};

class EventCovertHostages : public EventCovert {
public:
    CovertData d;
    EventCovertHostages(int16 id = -1, int16 target = -1, int16 success = 0);
    CovertData &cv() { return d; }
    uint8 *Data() { return (uint8 *)&d; }
    const char *ClassName();
    int16 Size() { return 0x14; }
    Event *Execute();
    void Display();
};

class EventCovertSetup : public EventCovert {
public:
    SetupData d;
    EventCovertSetup(int16 id = -1, int16 target = -1, int16 victim = -1, int16 success = 0);
    CovertData &cv() { return d; }
    uint8 *Data() { return (uint8 *)&d; }
    const char *ClassName();
    int16 Size() { return 0x16; }
    void OnEmpireDeleted(int16 id);
    int16 Due();
    Event *Execute();
    void Display();
};

// Investments in the bank or in bonds (vtable 375e).
class EventBank : public Event {
public:
    BankData d;
    EventBank(int16 id = -1, const char *name = "", int32 amount = 0, int16 year = 0,
              int16 issuer = -1);
    EventData &ev() { return d; }
    uint8 *Data() { return (uint8 *)&d; }
    const char *ClassName();
    int16 Size() { return 0x6a; }
    int16 Expired();
    void OnEmpireDeleted(int16 id);
    int16 Due();
    Event *Execute();
    void Display(int16 n);                      // 4c48:29cb
};

// A report about covert operations (vtable 3742).
class CovertReport : public ReportOut {
public:
    explicit CovertReport(int16 id) : ReportOut(id, COVERT_FILE) {}
};

int16 EventsEmpireDeleted(int16 id);            // 4c48:0274
void ClearEvents();                             // 4c48:0358
void ClearCovertReports();                      // 4c48:32a9
int16 ShowCovertReports(Player *p, Empire *e);  // 4c48:32bd
int16 ProcessEvents();                          // 4c48:32f1
int16 ListInvestments(int16 id);                // 4c48:35ce
int16 ListCovertOps(int16 id);                  // 4c48:375d
void AddEvent(Event *e);                        // 4c48:3b34
int32 BondPayoff(int16 issuer, int32 amount);   // 4c48:3bc9

#endif
