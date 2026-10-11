// Scheduled events: covert operations and bank investments (4c48).
#include "sre.h"
#include "events.h"
#include "battle.h"

const char *EVENT_FILE = "Data/Event";          // DS:303a
const char *COVERT_FILE = "Data/CovertOp";      // DS:308e
int32 g_nextevent;                              // DS:c95e

static const char *CLS_DISSENSION = "EventCovertDissension";    // DS:303e
static const char *CLS_DEMORALIZE = "EventCovertDemoralize";    // DS:304e
static const char *CLS_HOSTAGES = "EventCovertHostages";        // DS:305e
static const char *CLS_SETUP = "EventCovertSetup";              // DS:306e
static const char *CLS_BANK = "EventBank";                      // DS:307e

static Streamable *NewDissension() { return new EventCovertDissension(); }  // 4c48:04aa
static Streamable *NewDemoralize() { return new EventCovertDemoralize(); }  // 4c48:0b6f
static Streamable *NewHostages() { return new EventCovertHostages(); }      // 4c48:120e
static Streamable *NewSetup() { return new EventCovertSetup(); }            // 4c48:1afb
static Streamable *NewBank() { return new EventBank(); }                    // 4c48:28cb

// 4c48:4331
static struct EventRegs {
    ClassEntry e[5];
    EventRegs()
    {
        RegisterClass(&e[0], CLS_DISSENSION, NewDissension);
        RegisterClass(&e[1], CLS_DEMORALIZE, NewDemoralize);
        RegisterClass(&e[2], CLS_HOSTAGES, NewHostages);
        RegisterClass(&e[3], CLS_SETUP, NewSetup);
        RegisterClass(&e[4], CLS_BANK, NewBank);
    }
} s_eventregs;

static inline Empire &Emp(int16 id) { return g_empires[id]; }

// ---------------------------------------------------------------------
// Event

// 4c48:0000: expires after two weeks; due now.
void Event::Init(int16 id)
{
    EventData &e = ev();
    e.empire = id;
    e.created = g_now;
    e.expires = g_now + 0x127500;
    e.due = g_now;
}

// 4c48:011d
void Event::SetDue(int16 days, int16 hours, int16 minutes)
{
    ev().due = g_now + 86400 * (int32)days + 3600 * (int32)hours + 60 * (int32)minutes;
}

// 4c48:01b2
int16 Event::Expired()
{
    return g_now > ev().expires || ev().empire == -1;
}

// 4c48:0218
int16 Event::Due()
{
    return b_time(0) >= ev().due;
}

// ---------------------------------------------------------------------
// Covert operations

// 4c48:0392
void EventCovert::Init(int16 id, int16 target, int16 success)
{
    Event::Init(id);
    cv().target = target;
    cv().success = success;
}

// 4c48:046c
void EventCovert::OnEmpireDeleted(int16 id)
{
    if (cv().empire == id || cv().target == id)
        cv().empire = -1;
}

static void DisplayName(const char *s)
{
    kout << setw(20) << s << setw(0);
}

// 4c48:0425
EventCovertDissension::EventCovertDissension(int16 id, int16 target, int16 success)
{
    Init(id, target, success);
}
const char *EventCovertDissension::ClassName() { return CLS_DISSENSION; }   // 4c48:0492
void EventCovertDissension::Display() { DisplayName("Dissension"); }        // 4c48:0502

// 4c48:052e
Event *EventCovertDissension::Execute()
{
    Empire *a = &Emp(d.empire);
    Empire *t = &Emp(d.target);
    if (d.success) {
        ReportOut ra(a->id, 0);
        ReportOut rt(t->id, 0);
        int32 n = t->soldiers / ((int32)(((int32)7 * b_rand()) / 0x8000) + 9);
        rt << c_number << n << c_normal << " soldiers dissented from your army due to propaganda.";
        rt.SetTime(d.due);
        ra << "You successfully supported dissension of " << c_number << n << c_normal
           << " soldiers of the army of " << c_empire << Untr(t->name) << c_normal << '.';
        ra.SetTime(d.due);
        t->soldiers -= n;
        a->agents += 1;
    } else if (Exists(*t)) {
        ReportOut ra(a->id, 0);
        ReportOut rt(t->id, 0);
        ra << "You failed to cause dissension in " << c_empire << Untr(t->name) << c_normal << '.';
        ra.SetTime(d.due);
        rt << "A covert agent from " << c_empire << Untr(a->name) << c_normal
           << " was caught trying to support dissension and was executed!";
        rt.SetTime(d.due);
        if (r_failedcovert.Bool()) {
            g_log2 << c_empire << Untr(a->name) << c_normal
                   << " was caught trying to support dissension in " << Color(13)
                   << Untr(t->name) << c_normal << ".\n";
        }
    }
    return 0;
}

// 4c48:0aea
EventCovertDemoralize::EventCovertDemoralize(int16 id, int16 target, int16 success)
{
    Init(id, target, success);
}
const char *EventCovertDemoralize::ClassName() { return CLS_DEMORALIZE; }   // 4c48:0b57
void EventCovertDemoralize::Display() { DisplayName("Demoralize"); }        // 4c48:0bc7

// 4c48:0bf3
Event *EventCovertDemoralize::Execute()
{
    Empire *a = &Emp(d.empire);
    Empire *t = &Emp(d.target);
    if (d.success) {
        ReportOut ra(a->id, 0);
        ReportOut rt(t->id, 0);
        ra << "You succeeded in demoralizing troops of " << c_empire << Untr(t->name) << c_normal
           << '.';
        ra.SetTime(d.due);
        rt << "Mysterious shipments of alcohol demoralized your troops.";
        rt.SetTime(d.due);
        int8 drop = (int8)((int32)t->effectiveness /
                           ((int32)(((int32)7 * b_rand()) / 0x8000) + 7));
        t->effectiveness = (int8)(t->effectiveness - drop);
        if (t->effectiveness < 0)
            t->effectiveness = 0;
        if (t->effectiveness > 100)
            t->effectiveness = 100;
        a->agents += 1;
    } else if (Exists(*t)) {
        ReportOut ra(a->id, 0);
        ReportOut rt(t->id, 0);
        ra << "You failed to demoralize troops in " << c_empire << Untr(t->name) << c_normal << '.';
        ra.SetTime(d.due);
        rt << "A covert agent from " << c_empire << Untr(a->name) << c_normal
           << " was caught trying to demoralize troops and was executed!";
        rt.SetTime(d.due);
        if (r_failedcovert.Bool()) {
            g_log2 << c_empire << Untr(a->name) << c_normal
                   << " was caught trying to demoralize troops of " << Color(13)
                   << Untr(t->name) << c_normal << ".\n";
        }
    }
    return 0;
}

// 4c48:1189
EventCovertHostages::EventCovertHostages(int16 id, int16 target, int16 success)
{
    Init(id, target, success);
}
const char *EventCovertHostages::ClassName() { return CLS_HOSTAGES; }   // 4c48:11f6
void EventCovertHostages::Display() { DisplayName("Hostages"); }        // 4c48:1266

// 4c48:1292
Event *EventCovertHostages::Execute()
{
    Empire *a = &Emp(d.empire);
    Empire *t = &Emp(d.target);
    if (d.success) {
        int32 m = t->money.get() / ((int32)(((int32)3 * b_rand()) / 0x8000) + 5);
        int32 b = t->bank.get() / ((int32)(((int32)5 * b_rand()) / 0x8000) + 7);
        ReportOut ra(a->id, 0);
        ReportOut rt(t->id, 0);
        if (b < 0)
            b = 0;
        rt << "Terrorists took hostages and you had to pay " << c_number << (int32)(m + b)
           << c_normal << " credits as ransom.";
        rt.SetTime(d.due);
        ra << "Your covert agent succeeded in taking " << c_number << (int32)(m + b) << c_normal
           << " credits from " << c_empire << Untr(t->name) << c_normal
           << " as ransom for hostages.";
        ra.SetTime(d.due);
        t->money.sub(m);
        t->bank.sub(b);
        if (a->money.get() + b + m > 0) {
            a->money.add(b + m);
        } else {
            // Fill the treasury, then the bank, in steps.
            while (m + b > 10000) {
                if (a->money.get() < 2000000000)
                    a->money.add(10000);
                else
                    a->bank.add(10000);
                if (m > 10000)
                    m -= 10000;
                else if (b > 10000)
                    b -= 10000;
                else {
                    m += b;
                    b = 0;
                }
            }
            a->money.add(m + b);
        }
        a->agents += 1;
    } else if (Exists(*t)) {
        ReportOut ra(a->id, 0);
        ReportOut rt(t->id, 0);
        ra << "You failed to take hostages in " << c_empire << Untr(t->name) << c_normal << ".\n";
        ra.SetTime(d.due);
        rt << "A covert agent from " << c_empire << Untr(a->name) << c_normal
           << " was caught trying to take hostages and " << RandomPronoun(0) << " was executed!";
        rt.SetTime(d.due);
        if (r_failedcovert.Bool()) {
            g_log2 << c_empire << Untr(a->name) << c_normal
                   << " was caught trying to take hostages from " << Color(13)
                   << Untr(t->name) << c_normal << ".\n";
        }
    }
    return 0;
}

// 4c48:1a60
EventCovertSetup::EventCovertSetup(int16 id, int16 target, int16 victim, int16 success)
{
    Init(id, target, success);
    d.victim = victim;
}
const char *EventCovertSetup::ClassName() { return CLS_SETUP; }     // 4c48:1ae3
void EventCovertSetup::Display() { DisplayName("Setup"); }          // 4c48:1b9d

// 4c48:1ab1
void EventCovertSetup::OnEmpireDeleted(int16 id)
{
    if (d.empire == id || d.target == id || d.victim == id)
        d.empire = -1;
}

// 4c48:1b57: only while the target is playing.
int16 EventCovertSetup::Due()
{
    if (!g_current)
        return 0;
    return Event::Due() && d.target == g_current->id;
}

// 4c48:1bc9: make the victim believe the target attacked it.
Event *EventCovertSetup::Execute()
{
    Empire *a = &Emp(d.empire);
    Empire *t = &Emp(d.target);
    Empire *v = &Emp(d.victim);
    if (d.success) {
        CovertReport ra(a->id);
        ra << "Your covert agent was successful in setting up " << c_empire << Untr(t->name)
           << c_normal << " and " << c_empire << Untr(v->name) << c_normal << '.';
        BattleReport rv(v->id);
        int16 sev = (int16)(-(int16)(((int32)5 * b_rand()) / 0x8000) - 1);
        AttackMessage(rv, t->name, sev);
        AttackNews(t, v, 0, sev);
        SetTreaty(t->treaty[v->id], TT_NONE, 0);
        t->treaty[v->id].setstatus(TS_NONE);
        SetTreaty(v->treaty[t->id], TT_NONE, 0);
        v->treaty[t->id].setstatus(TS_NONE);
        a->agents += 1;
    } else if (Exists(*t) && Exists(*v)) {
        CovertReport ra(a->id);
        ra << "Your covert agent was caught setting up " << c_empire << Untr(t->name) << c_normal
           << " and " << c_empire << Untr(v->name) << c_normal << ", and " << RandomPronoun(0)
           << " was executed.";
        if (t->id != a->id) {
            CovertReport r(t->id);
            r << "A covert agent from " << c_empire << Untr(a->name) << c_normal
              << " was caught trying to set you up.";
        }
        if (v->id != a->id) {
            CovertReport r(v->id);
            r << "A covert agent from " << c_empire << Untr(a->name) << c_normal
              << " was caught trying to make you think " << Untr(t->name) << " attacked you.";
        }
        if (r_failedcovert.Bool()) {
            g_log2 << c_empire << Untr(a->name) << c_normal << " attempted to set up "
                   << Color(13) << Untr(t->name) << c_normal << " and " << Color(13)
                   << Untr(v->name) << c_normal << ".\n";
        }
    }
    return 0;
}

// ---------------------------------------------------------------------
// Bank investments

// 4c48:2773
EventBank::EventBank(int16 id, const char *name, int32 amount, int16 year, int16 issuer)
{
    Init(id);
    d.amount.v = 0;
    d.m14.v = 0;
    d.amount.set(0);
    memset(d.name, 0, sizeof d.name);
    strcpy(d.name, name);
    d.year = year;
    d.amount.set(amount);
    d.issuer = issuer;
}

const char *EventBank::ClassName() { return CLS_BANK; }     // 4c48:28b3

// 4c48:2962
int16 EventBank::Expired()
{
    return d.amount.get() <= 0 || d.empire == -1;
}

// 4c48:284e
void EventBank::OnEmpireDeleted(int16 id)
{
    if (d.empire == id) {
        if (d.issuer != -1)
            Emp(d.issuer).bonds--;
        d.empire = -1;
    }
    if (d.issuer == id) {
        d.issuer = -1;
        d.amount.set(0);
    }
}

// 4c48:298d
int16 EventBank::Due()
{
    return Event::Due() && Emp(d.empire).year >= d.year;
}

// 4c48:29cb
void EventBank::Display(int16 n)
{
    if (n == 1) {
        kout << Color(10) << "###" << c_number << setwr(14) << "Credits" << setwr(7) << Color(15)
             << "Turn" << setwr(7) << Color(15) << "Hours" << setw(0) << '\n';
        HLine(Color(3), 0x4b);
    }
    kout << Color(10) << setw(3) << n << setw(0) << c_number << setw(14) << d.amount.get()
         << setw(0) << Color(15) << setw(7) << d.year;
    int16 hours = (int16)((d.due - b_time(0) + 1800) / 3600);
    if (hours < 0)
        hours = 0;
    kout << setw(7) << hours << setw(0) << c_normal << "  " << d.name << '\n';
}

// 4c48:2c42: the investment matures into the bank (or the treasury when
// the bank is full); bonds are paid by their issuer.
Event *EventBank::Execute()
{
    Empire *e = &Emp(d.empire);
    int32 original = d.amount.get();
    int32 paid = 0;
    if (e->bank.get() < 2000000000 && e->bank.get() + d.amount.get() > 0) {
        e->bank.add(d.amount.get());
        paid = d.amount.get();
        d.amount.set(0);
    } else {
        while (e->bank.get() < 2000000000 && d.amount.get() > 10000000) {
            e->bank.add(10000000);
            d.amount.sub(10000000);
            paid += 10000000;
        }
        while (e->bank.get() < 2000000000 && d.amount.get() > 10000) {
            e->bank.add(10000);
            d.amount.sub(10000);
            paid += 10000;
        }
        if (e->money.get() < 2000000000 && e->money.get() + d.amount.get() > 0) {
            e->money.add(d.amount.get());
            paid += d.amount.get();
            d.amount.set(0);
        }
    }
    Event *next = 0;
    int32 actual = BondPayoff(d.issuer, paid);
    ReportOut r(d.empire, 0);
    r << "Your " << c_input << d.name << c_normal;
    if (original == 0) {
        r << " have crashed!";
    } else if (actual < paid) {
        r << " have crashed, giving you only " << c_number << actual << c_normal
          << " credits in the bank.";
        if (e->bank.get() >= paid)
            e->bank.sub(paid - actual);
        else if (e->money.get() >= paid - actual)
            e->money.sub(paid - actual);
    } else {
        r << " have returned your investment, giving you " << c_number << paid << c_normal
          << " credits in the bank.";
    }
    if (d.amount.get() > 0) {
        r << "  " << c_number << d.amount.get() << c_normal
          << " credits have been re-invested because your bank was full.";
        EventBank *b = new EventBank(d.empire, d.name, d.amount.get(), d.year, d.issuer);
        b->SetDue(0, 24, 0);
        next = b;
    } else if (d.issuer > -1 && Exists(Emp(d.issuer))) {
        Emp(d.issuer).bonds--;
    }
    r.SetTime(d.due);
    if (paid <= 0)
        r.rep.ClearRecipients();
    return next;
}

// ---------------------------------------------------------------------

// 4c48:0274
int16 EventsEmpireDeleted(int16 id)
{
    int16 n = 0;
    StreamFile f(EVENT_FILE);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        o->OnEmpireDeleted(id);
        n++;
        f.Keep(o);
        delete o;
    }
    return n;
}

// 4c48:0358
void ClearEvents()
{
    StreamFile f(EVENT_FILE);
}

// 4c48:32a9
void ClearCovertReports()
{
    ClearStreamFile(COVERT_FILE);
}

// 4c48:32bd
int16 ShowCovertReports(Player *p, Empire *e)
{
    return ShowReports(e->id, COVERT_FILE, HasPref(p->prefs2, PF2_REPORTDATES), 5, 13);
}

// 4c48:32f1: run the events that are due.
int16 ProcessEvents()
{
    if (b_time(0) < g_nextevent)
        return 0;
    GalaxyOpen();
    int16 n = 0;
    {
        StreamFile f(EVENT_FILE);
        g_nextevent = 0x7fffffff;
        for (;;) {
            Streamable *o = f.Read();
            if (!o)
                break;
            Event *ev = (Event *)o;
            if (ev->ev().due < g_nextevent)
                g_nextevent = ev->ev().due;
            if (ev->Due()) {
                n++;
                Event *nx = ev->Execute();
                if (nx) {
                    if (nx->ev().due < g_nextevent)
                        g_nextevent = nx->ev().due;
                    f.Keep(nx);
                    delete nx;
                }
            } else {
                if (ev->ev().due < g_nextevent)
                    g_nextevent = ev->ev().due;
                f.Keep(ev);
            }
            delete ev;
        }
        GalaxyClose();
    }
    return n;
}

// 4c48:35ce
int16 ListInvestments(int16 id)
{
    int16 n = 0;
    StreamFile f(EVENT_FILE);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        if (StrEqI(CLS_BANK, String(o->ClassName())) && ((Event *)o)->ev().empire == id)
            ((EventBank *)o)->Display(++n);
        f.Keep(o);
        delete o;
    }
    if (n > 0)
        HLine(Color(3), 0x4b);
    else
        kout << c_normal << "You have no investments.\n";
    return n;
}

// 4c48:375d
int16 ListCovertOps(int16 id)
{
    int16 n = 0;
    StreamFile f(EVENT_FILE);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        if (((Event *)o)->ev().empire == id) {
            String cls(o->ClassName());
            if (StrEqI(CLS_DISSENSION, cls) || StrEqI(CLS_DEMORALIZE, cls) ||
                StrEqI(CLS_SETUP, cls) || StrEqI(CLS_HOSTAGES, cls)) {
                EventCovert *c = (EventCovert *)o;
                n++;
                if (n == 1)
                    HLine(Color(5), 0x4b);
                kout << Color(13) << setw(5) << n << setw(0) << Color(15) << ' ';
                c->Display();
                if (c->cv().target != -1)
                    kout << c_normal << " to " << Untr(Emp(c->cv().target).name);
                kout << '\n';
            }
        }
        f.Keep(o);
        delete o;
    }
    if (n > 0)
        HLine(Color(5), 0x4b);
    else
        kout << c_normal << "You have no covert agents out on missions.\n";
    return n;
}

// 4c48:3b34
void AddEvent(Event *e)
{
    if (!e)
        return;
    if (e->ev().due < g_nextevent)
        g_nextevent = e->ev().due;
    AppendObject(EVENT_FILE, e);
}

// 4c48:3bc9: the issuer of bonds pays for them, selling assets if it
// has to; returns what was actually paid.
int32 BondPayoff(int16 issuer, int32 amount)
{
    Money owed;
    owed.v = amount;
    int16 sold = 0;
    if (issuer == -1)
        return amount;
    if (Exists(Emp(issuer))) {
        Empire *e = &Emp(issuer);
        if (e->money.get() >= owed.get()) {
            e->money.sub(owed.get());
            owed.set(0);
        } else {
            owed.sub(e->money.get());
            e->money.set(0);
        }
        if (e->bank.get() >= owed.get()) {
            e->bank.sub(owed.get());
            owed.set(0);
        } else {
            owed.sub(e->bank.get());
            e->bank.set(0);
        }
        while (owed.get() > 0) {
            int32 before = owed.get();
            sold = 1;
            if (owed.get() > 0 && e->soldiers > 100) {
                owed.sub(100 * (PriceSoldiers(e) / 3));
                e->soldiers -= 100;
            }
            if (owed.get() > 0 && e->fighters > 100) {
                owed.sub(100 * (PriceFighters(e) / 3));
                e->fighters -= 100;
            }
            if (owed.get() > 0 && e->stations > 100) {
                owed.sub(100 * (PriceStations(e) / 3));
                e->stations -= 100;
            }
            if (owed.get() > 0 && e->generals > 10) {
                owed.sub(10 * (PriceGenerals(e) / 3));
                e->generals -= 10;
            }
            if (owed.get() > 0 && e->heavycruisers > 100) {
                owed.sub(100 * (PriceHeavyCruisers(e) / 3));
                e->heavycruisers -= 100;
            }
            if (owed.get() > 0 && e->carriers > 10) {
                owed.sub(10 * (PriceCarriers(e) / 3));
                e->carriers -= 10;
            }
            if (owed.get() > 0 && e->agents > 10) {
                owed.sub(10 * (PriceAgents(e) / 3));
                e->agents -= 10;
            }
            if (owed.get() > 0 && e->food > 100) {
                owed.sub(200);
                e->food -= 100;
            }
            if (owed.get() > 0 && e->population > 100) {
                owed.sub(100);
                e->population -= 10;
            }
            for (int16 i = 0; i < 10; i++) {
                if (owed.get() > 0 && Planets(e, i)->count > 10) {
                    owed.sub(PricePlanet(e) / 3);
                    Planets(e, i)->count -= 1;
                }
            }
            if (before == owed.get())
                break;
        }
        ReportOut r(issuer, 0);
        r << c_number << amount << c_normal
          << " credits were taken from you to pay for for bond payoffs.";
        if (sold)
            r << "  Some of your assets had to be sold to get cash for this expense.";
    }
    if (owed.get() > 0)
        return amount - owed.get();
    return amount;
}
