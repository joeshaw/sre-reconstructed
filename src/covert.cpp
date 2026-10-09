// Covert operations (segment 409c).
#include "sre.h"
#include "events.h"

void ViewRelationsOf(Empire *e);                       // 70d2:1787

static const char *CAUGHT = "Your covert agent was caught and was publicly executed!\n";

// 14f6:05dc: the mask of empires an operation was done to this turn
// (or today).
int32 *OpMask(Empire *e, int16 op)
{
    switch (op) {
    case 1: return &e->d22f[0];
    case 2: return &e->d22f[6];
    case 3: return &e->d22f[1];
    case 4: return &e->d22f[2];
    case 5: return &e->d22f[3];
    case 7: return &e->d22f[4];
    case 8: return &e->d22f[5];
    }
    return (int32 *)(e->u_24b + 0x20);
}

static int16 Alive(Empire *e)
{
    return Exists(*e) && TotalPlanets(e) > 0 && e->population > 0;
}

// 409c:0000: the odds of a covert operation.
static const char *Chance(Empire *me, Empire *t)
{
    int32 a = me->agents * (TotalPlanets(t) + 300);
    int32 b = t->agents * (TotalPlanets(me) + 300);
    int32 a0 = a;
    a += b / 5;
    b += a0 / 5;
    while (a > 1 && b > 1) {
        a >>= 1;
        b >>= 1;
    }
    if (a == b)
        return "FAIR";
    if (a <= 1) {
        if (b > 15)
            return "NONEXISTENT";
        if (b > 7)
            return "VERY SLIM";
        if (b > 3)
            return "SLIM";
        return "UNFAVORABLE";
    }
    if (a > 15)
        return "ALMOST CERTAIN";
    if (a > 7)
        return "FAIRLY GOOD";
    if (a > 3)
        return "GOOD";
    return "FAVORABLE";
}

// 409c:0197: does a covert operation of me against t succeed?
int16 SpySucceeds(Empire *me, Empire *t)
{
    if (me == t)
        return 1;
    int32 a = me->agents * (TotalPlanets(t) + 300);
    int32 b = t->agents * (TotalPlanets(me) + 300);
    int32 a0 = a;
    a += b / 5;
    b += a0 / 5;
    while (a > 32000 || b > 32000) {
        a /= 2;
        b /= 2;
    }
    uint16 ea = (uint16)a, eb = (uint16)b;
    if (ea == 0)
        return 0;
    if (eb == 0)
        return 1;
    uint16 x = (uint16)(ea + (uint16)(((int32)(int16)eb * b_rand()) / 0x8000));
    uint16 y = (uint16)(eb + (uint16)(((int32)(int16)ea * b_rand()) / 0x8000));
    return x > y;
}

// 409c:0508: a covert operation ends the treaties between two empires.
static void BreakTreaties(Empire *me, Empire *t)
{
    SetTreaty(me->treaty[t->id], 0, 0);
    me->treaty[t->id].setc(0);
    SetTreaty(t->treaty[me->id], 0, 0);
    t->treaty[me->id].setc(0);
}

// 409c:033d
static int16 ChooseTarget(Player *p, Empire *me, int16 op)
{
    int32 allowed = EmpireMask() & ~ProtectedMask();
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (!Alive(&g_empires[i]))
            MaskClear(&allowed, i);
    if (op != 2)
        MaskClear(&allowed, me->id);
    if (op != 0 && op != 6) {
        for (int16 i = 0; i < MAXEMPIRES; i++) {
            Treaty t = me->treaty[i];
            if ((t.type() != 0 && t.a > 0) || t.type() == 1)
                MaskClear(&allowed, i);
        }
    }
    int16 id = ChooseEmpire(allowed);
    if (id == -1)
        return -1;
    if (op != 0 && op != 6 && op != 8 && MaskTest(*OpMask(me, op), id)) {
        kout << "But you have already performed covert ops with that empire!  Limit one empire per ";
        kout << ((op == 1 || op == 3 || op == 4) ? "turn.\n" : "day.\n");
        return -1;
    }
    return id;
}

// 409c:084d: pay for an operation and choose its target.
static int16 Attempt(Player *p, Empire *me, int32 cost, int16 *target, int16 op)
{
    if (me->money.get() < cost) {
        kout << "You cannot afford that.\n";
        return 0;
    }
    *target = ChooseTarget(p, me, op);
    if (*target == -1)
        return 0;
    if (op != 1 && op != 0 && op != 8) {
        kout << c_normal << "Chance of success:  " << Color(13)
             << Chance(me, &g_empires[*target]) << c_normal << ".\n";
        if (!YesNo(1, "Do you want to continue"))
            return 0;
    }
    if (op != 0 && op != 6 && op != 8)
        MaskSet(OpMask(me, op), *target);
    me->money.sub(cost);
    return 1;
}

// 409c:09b3
struct CovertPrompt : MenuHook {
    Empire *e;
    explicit CovertPrompt(Empire *x) : e(x) {}
    int16 Call()
    {
        kout << "You have " << c_number << e->money.get() << c_normal << " credits and "
             << c_number << e->agents << c_normal << " covert agent";
        if (e->agents != 1)
            kout << 's';
        kout << ".\nWhat would you like to do?  ";
        return 0;
    }
};

static void Caught(Empire *me, Empire *t, const char *what)
{
    CovertReport r(t->id);
    r << "A covert agent from " << c_empire << Untr(me->name) << c_normal << what;
}

// 409c:0a6d
void CovertOps(Player *p, Empire *me, Galaxy *g)
{
    int32 cost[10];
    cost[1] = me->year < 0 ? 880 : 3100;
    cost[2] = 10000;
    cost[3] = 20000;
    cost[4] = 25000;
    cost[5] = 30000;
    cost[6] = 30000;
    cost[7] = 30000;
    cost[8] = 50000;
    cost[9] = 50000;
    while (me->agents > 0) {
        Menu m("Covert", "Covert Operations", 0xe, 5, 5, 0xe, 0xd, 0xf, 7, 7, 7, 7);
        m.prompt = new CovertPrompt(me);
        m.Add("Send spy", '1', 1);
        if (me->year >= 0) {
            m.Add("Insurgent aid", '2', 2);
            m.Add("Set up", '3', 3);
            m.Add("Support dissension", '4', 4);
            m.Add("Demoralize troops", '5', 5);
            m.Add("Bombing operations", '6', 6);
        }
        m.Add("Infiltration", '7', 7);
        if (me->year >= 0) {
            if (g_system.regstate)
                m.Add("Take hostages", '8', 8);
            m.Add("Bribe spy", '9', 9);
        }
        if (g_system.regstate)
            m.Add("", 'V', 0x50);
        m.Add("", 'S', 0x51);
        for (int16 i = 1; i <= 9; i++)
            if (m.Find(i))
                m.Find(i)->AddColumn(cost[i]);
        int16 r = m.Run(1, -1);
        int16 id;
        Empire *t;
        switch (r) {
        case 1:
            if (!Attempt(p, me, cost[1], &id, 0))
                break;
            t = &g_empires[id];
            if (SpySucceeds(me, t) || MaskTest(*OpMask(me, 8), t->id)) {
                kout << "Your spy was successful.\nSpy results are...\n";
                EmpireStatus(t, 0);
            } else {
                CovertReport rep(t->id);
                kout << CAUGHT;
                rep << "A covert agent from " << c_empire << Untr(me->name) << c_normal
                    << " was caught spying on your empire and was terminated!";
                me->agents -= 1;
            }
            break;
        case 2: {
            if (!Attempt(p, me, cost[2], &id, 1))
                break;
            t = &g_empires[id];
            BreakTreaties(me, t);
            if (t->insurgency >= 7) {
                kout << "That empire already is " << c_number << InsurgencyName(t) << c_normal
                     << ".\n";
                break;
            }
            MaskSet(OpMask(me, 1), id);
            int16 k = 0;
            while (k < t->insurgency) {
                if (!SpySucceeds(me, t)) {
                    CovertReport rep(t->id);
                    rep << "A covert agent from " << c_empire << Untr(me->name) << c_normal
                        << " was caught trying to aid insurgents and was executed!";
                    me->agents -= 1;
                    if (r_failedcovert.Bool())
                        g_log2 << c_empire << Untr(me->name) << c_normal
                               << " attempted to aid insurgents in " << Color(13)
                               << Untr(t->name) << c_normal << "!\n";
                    kout << CAUGHT;
                    break;
                }
                k++;
            }
            if (t->insurgency == k) {
                InsurgencyUp(t);
                kout << "Your spy was successful in raising " << c_number << Untr(t->name) << "'s"
                     << c_normal << " insurgency to " << c_number << InsurgencyName(t) << c_normal
                     << ".\n";
            }
            break;
        }
        case 3: {
            if (me->money.get() < cost[3]) {
                kout << "You can't afford to set anyone up.\n";
                break;
            }
            kout << "Choose an 'attacker' empire:\n";
            int16 a = ChooseTarget(p, me, 2);
            if (a == -1)
                break;
            kout << "Choose a 'defender' empire:\n";
            int16 d = ChooseTarget(p, me, 2);
            if (d == -1)
                break;
            if (a == d) {
                kout << "That makes no sense.\n";
                break;
            }
            me->money.sub(cost[3]);
            Empire *ea = &g_empires[a], *ed = &g_empires[d];
            BreakTreaties(me, ea);
            BreakTreaties(me, ed);
            MaskSet(OpMask(me, 2), a);
            MaskSet(OpMask(me, 2), d);
            me->agents -= 1;
            int16 ok = SpySucceeds(me, ea) || SpySucceeds(me, ed);
            {
                EventCovertSetup ev(me->id, ea->id, ed->id, ok);
                AddEvent(&ev);
            }
            kout << "You have sent an agent to set up " << Color(14) << Untr(ea->name) << c_normal
                 << " and " << Color(11) << Untr(ed->name) << c_normal << ".\n";
            break;
        }
        case 4: {
            if (!Attempt(p, me, cost[4], &id, 3))
                break;
            t = &g_empires[id];
            BreakTreaties(me, t);
            me->agents -= 1;
            MaskSet(OpMask(me, 3), id);
            int16 ok = SpySucceeds(me, t);
            {
                EventCovertDissension ev(me->id, t->id, ok);
                ev.SetDue(0, 12, 0);
                AddEvent(&ev);
            }
            kout << "You have sent a covert agent to " << c_number << Untr(t->name) << c_normal;
            kout << " to support dissension.\n";
            break;
        }
        case 5: {
            if (!Attempt(p, me, cost[5], &id, 4))
                break;
            t = &g_empires[id];
            me->agents -= 1;
            BreakTreaties(me, t);
            MaskSet(OpMask(me, 4), id);
            int16 ok = SpySucceeds(me, t);
            {
                EventCovertDemoralize ev(me->id, t->id, ok);
                ev.SetDue(0, 12, 0);
                AddEvent(&ev);
            }
            kout << "You have sent a covert agent to " << c_number << Untr(t->name) << c_normal;
            kout << " to demoralize troops.\n";
            break;
        }
        case 6:
            if (!Attempt(p, me, cost[6], &id, 5))
                break;
            t = &g_empires[id];
            BreakTreaties(me, t);
            MaskSet(OpMask(me, 5), id);
            if (!SpySucceeds(me, t)) {
                CovertReport rep(t->id);
                me->agents -= 1;
                rep << "A covert agent from " << c_empire << Untr(me->name) << c_normal
                    << " was caught trying to destroy your food supplies and was executed!";
                if (Random(3) == 0)
                    kout << "Your covert agent was unsuccessful, and now " << RandomPronoun(0)
                         << "'s dead.\n";
                else if (Random(2) == 0)
                    kout << "Your covert agent accidentally blew " << RandomPronoun(1)
                         << "self up.\n";
                else
                    kout << "Your covert agent was unsuccessful, and " << RandomPronoun(0)
                         << " was electrocuted.\n";
            } else {
                {
                    CovertReport rep(t->id);
                    rep << "A bomb blew up food supplies!";
                }
                if (Random(3) == 0) {
                    t->food -= t->food / 2;
                    kout << "You destroyed some";
                } else {
                    t->food = 0;
                    if (Random(8) == 0 && ShowScreen("Screen/", "CovBomb", 0))
                        Pause();
                    kout << "You successfully destroyed all";
                }
                kout << " of the enemy's food supplies.\n";
            }
            break;
        case 7:
            for (;;) {
                Menu im("Infiltration", "Infiltration Ops", 0xe, 5, 5, 0xe, 0xd, 0xf, 7, 7, 7, 7);
                im.prompt = new CovertPrompt(me);
                im.Add("Intercept Messages", '1', 1);
                im.Add("Diplomatic Status", '2', 2);
                int16 c = im.Run(1, -1);
                if (c == -1)
                    break;
                if (c == 1) {
                    if (!Attempt(p, me, cost[7], &id, 6))
                        continue;
                    t = &g_empires[id];
                    me->agents -= 1;
                    if ((uint8)t->b2d3[me->id] == 0xff || !SpySucceeds(me, t)) {
                        CovertReport rep(t->id);
                        rep << "A covert agent from " << c_empire << Untr(me->name) << c_normal
                            << " was caught trying to intercept your messages and "
                            << RandomPronoun(0) << " was killed!";
                        kout << CAUGHT;
                    } else {
                        kout << "Your spy was has been planted.\n";
                        t->b2d3[me->id]++;
                    }
                } else if (c == 2) {
                    if (!Attempt(p, me, cost[7], &id, 6))
                        continue;
                    t = &g_empires[id];
                    if (!SpySucceeds(me, t)) {
                        CovertReport rep(t->id);
                        rep << "A covert agent from " << c_empire << Untr(me->name) << c_normal
                            << " was caught trying to discover relations of your empire and "
                            << RandomPronoun(0) << " was killed!";
                        me->agents -= 1;
                        kout << CAUGHT;
                    } else {
                        kout << "Your spy was successful.\nSpy results are...\n";
                        ViewRelationsOf(t);
                    }
                }
            }
            break;
        case 8: {
            if (!Attempt(p, me, cost[8], &id, 7))
                break;
            t = &g_empires[id];
            BreakTreaties(me, t);
            MaskSet(OpMask(me, 7), id);
            me->agents -= 1;
            int16 ok = SpySucceeds(me, t);
            {
                EventCovertHostages ev(me->id, t->id, ok);
                ev.SetDue(0, 12, 0);
                AddEvent(&ev);
            }
            kout << "You have sent a covert agent to " << c_number << Untr(t->name) << c_normal;
            kout << " to take hostages.\n";
            break;
        }
        case 9: {
            if (!Attempt(p, me, cost[9], &id, 8))
                break;
            t = &g_empires[id];
            if (MaskTest(*OpMask(me, 8), t->id)) {
                me->money.add(cost[9]);
                kout << "You already have a spy working for you in that empire!\n";
                break;
            }
            int32 bribed = 0;
            for (int16 i = 0; i < MAXEMPIRES; i++)
                if (MaskTest(*OpMask(&g_empires[i], 8), t->id))
                    bribed += 1;
            if (t->agents <= bribed) {
                me->money.add(cost[9]);
                kout << "That empire has no agents to bribe!\n";
                break;
            }
            int32 avail = t->agents;
            for (int16 i = 0; i < MAXEMPIRES; i++)
                avail -= MaskTest(*OpMask(&g_empires[i], 8), t->id);
            if (SpySucceeds(me, t) && SpySucceeds(me, t) && avail > 0) {
                MaskSet(OpMask(me, 8), t->id);
                kout << "You have successfully bribed an agent from " << c_empire << Untr(t->name)
                     << c_normal << ".\n";
            } else {
                CovertReport rep(t->id);
                rep << "A covert agent from " << c_empire << Untr(me->name) << c_normal
                    << " was caught trying to bribe your covert agent and " << RandomPronoun(0)
                    << " was shot!";
                me->agents -= 1;
                kout << "Your covert agent was executed!\n";
            }
            break;
        }
        case 0x50:
            Bank(p, me, &g_galaxy);
            break;
        case 0x51:
            ListCovertOps(me->id);
            break;
        }
        if (r == -1)
            break;
    }
}
