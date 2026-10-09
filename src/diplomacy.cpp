// Diplomacy: proposing, accepting and breaking treaties (segment 70d2)
// and the treaty names (14f6:044e, 14f6:04ba).
//
// e->treaty[o] is e's treaty with o.  Its state field c records a
// message from o to e: 1 = o proposes this treaty, 2 = e rejected o's
// proposal, 3 = o accepted e's proposal, 4 = o breaks the treaty.
#include "sre.h"

Resource r_diplomacy("Diplomacy", (int32)1);                 // b01e
static Resource r_neutral("Treaty.Neutrality", (int32)1);    // b02a
static Resource r_freetrade("Treaty.FreeTrade", (int32)1);   // b036
static Resource r_minor("Treaty.MinorAlliance", (int32)1);   // b042
static Resource r_armed("Treaty.ArmedDefense", (int32)1);    // b04e
static Resource r_cruiser("Treaty.CruiserDefense", (int32)1);  // b05a
static Resource r_total("Treaty.TotalDefense", (int32)1);    // b066
static Resource r_break("Treaty.Break", (int32)1);           // b072
static Resource r_custom("Treaty.Custom", (int32)1);         // b07e

// 14f6:044e
String TreatyName(int16 type)
{
    static const char *const NAMES[8] = {
        "No Relations", "Neutrality/Peace Treaty", "Free Trade Agreement", "Minor Alliance Pact",
        "Armed Defense Pact", "Cruiser Protection Plan", "Total Defense Pact", "Custom Treaty"
    };
    if ((uint16)type > 7)
        return String("#<unknown-treaty-type>");
    return String(NAMES[type]);
}

// 14f6:04ba: the name, with the percentages of a custom treaty.
String TreatyDesc(Treaty t)
{
    String s = TreatyName(t.type());
    if (t.type() != 7)
        return s;
    return s + " (" + IntStr(t.e()) + "%/" + IntStr(t.d()) + "%)";
}

static int16 Alive(Empire *e)
{
    return Exists(*e) && TotalPlanets(e) > 0 && e->population > 0;
}

// 70d2:0399
struct DiplomacyPrompt : MenuHook {
    int16 Call()
    {
        kout << c_normal << "What would you like to do?  ";
        return 0;
    }
};

// 70d2:17b1: the treaties of an empire with the others; the empires in
// mark are bracketed.
static void Relations(Empire *me, int32 mark)
{
    kout << Color(14) << "\nRelations with other empires\n" << c_normal;
    HLine(Color(2), 0x46);
    GalaxyOpen();
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (kin.KeyReady()) {
            GalaxyClose();
            return;
        }
        if (me->id == i)
            continue;
        Empire *o = &g_empires[i];
        if (!Exists(*o))
            continue;
        char letter = o->id == -1 ? 0 : (char)(o->id + 'A');
        if (MaskTest(mark, i))
            kout << Color(2) << '[' << Color(14) << letter << Color(2) << ']';
        else
            kout << Color(7) << ' ' << letter << ' ';
        kout << "   " << Color(15) << setw(0x20) << Untr(o->name) << setw(0);
        Treaty mine = me->treaty[o->id];
        Treaty *theirs = &o->treaty[me->id];
        if (mine.type() != 0) {
            kout << c_number << TreatyDesc(mine);
            if (mine.a > 0)
                kout << ' ' << c_bracket << '[' << c_bold << (int16)mine.a << c_bracket << ']'
                     << c_normal;
        } else if (theirs->type() != 0) {
            kout << Color(10) << TreatyDesc(*theirs) << ' ';
            switch (theirs->c()) {
            case 1: kout << "Proposed"; break;
            case 2: kout << "Rejected"; break;
            case 3: kout << "Accepted"; break;
            case 4: kout << "Treaty Broken"; break;
            }
        } else {
            kout << c_normal << TreatyName(0);
        }
        kout << '\n';
    }
    GalaxyClose();
    HLine(Color(2), 0x46);
    kout << '\n';
}

// 70d2:09f5
static void RelationsList(int32 mask)
{
    Relations(g_current, mask);
}

// 70d2:03bd: may me propose this treaty to o?
static int16 CheckProposal(Empire *o, int16 meid, int16 type, int16 days)
{
    int16 shown = 0;
    Treaty *t = &o->treaty[meid];
    if (t->c() == 2) {
        kout << "You just rejected " << Untr(o->name) << "'s treaty.  ";
        if (!YesNo(0, "Do you wish to send a new treaty instead"))
            return 0;
    }
    if (o->id == g_galaxy.weaponsmaster && type != 1 && type != 2) {
        kout << Untr(o->name)
             << " is the Weapons Master; you cannot get a defense treaty with today's Weapons Master.\n";
        return 0;
    }
    if (t->c() == 1) {
        kout << "You have already proposed a " << c_number << TreatyDesc(*t) << c_normal;
        if (t->a > 0)
            kout << " for " << (int16)t->a << " turns";
        kout << " to " << Untr(o->name) << "!\n";
        shown = 1;
    }
    if (t->c() == 4) {
        kout << "You have sent word that you would like to break a treaty!\n";
        shown = 1;
    }
    if (!shown) {
        Treaty *mine = &g_empires[meid].treaty[o->id];
        if (mine->type() == type && mine->a >= days) {
            kout << "You already have a " << c_number << TreatyDesc(*mine) << c_normal;
            if (mine->a > 0)
                kout << " for " << (int16)mine->a << " turns";
            kout << " with " << Untr(o->name) << "!\n";
            return 0;
        }
    }
    if (shown && !YesNo(0, "Replace previous actions"))
        return 0;
    return 1;
}

// 70d2:0a10
static void Propose(Player *p, Empire *me, int16 type)
{
    if (type != 1 && type != 2 && me->id == g_galaxy.weaponsmaster) {
        kout << "You cannot get a defense treaty while you are the Weapons Master!\n";
        return;
    }
    Treaty t = { 0, 0, 0 };
    SetTreaty(t, type, 0);
    if (t.d() > 0 || t.e() > 0)
        kout << "This treaty will provide " << t.e() << "% soldier defense and " << t.d()
             << "% cruiser defense.\n";
    int16 days = -1, soldiers = 0, cruisers = 0;
    int32 chosen = 0;
    int32 allowed = EmpireMask();
    MaskClear(&allowed, me->id);
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (!Alive(&g_empires[i]))
            MaskClear(&allowed, i);
    if (ChooseEmpires(allowed, &chosen, String(""), RelationsList, 0) != 1)
        return;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (!MaskTest(chosen, i))
            continue;
        Empire *o = &g_empires[i];
        if (days == -1) {
            kout << "How many days?  " << c_bracket << '[' << c_bold << "MAX=" << (int16)14
                 << c_bracket << ']' << c_normal << "  ";
            kin >> defval(0) >> maxval(14) >> days;
            if (type == 7) {
                kout << "In a Custom Treaty, you may set the percentage of soldiers and ";
                kout << "heavy cruisers that are sent in the event of your treaty partner ";
                kout << "being attacked.  ";
                kout << "The total percentage of cruisers and soldiers can be up to 60%.\n";
                do {
                    kout << "Percentage of soldiers to send:  ";
                    kin >> defval(10) >> maxval(60) >> soldiers;
                    kout << "Percentage of cruisers to send:  ";
                    kin >> defval(10) >> maxval(60 - soldiers) >> cruisers;
                } while (cruisers + soldiers > 60);
            }
        }
        if (!CheckProposal(o, me->id, type, days))
            continue;
        kout << Color(10) << (char)(i + 'A') << ": " << c_normal;
        Treaty *ot = &o->treaty[me->id];
        SetTreaty(*ot, type, (uint8)days);
        ot->setc(1);
        if (type == 7) {
            ot->settype(7);
            ot->a = (uint8)days;
            ot->sete(soldiers);
            ot->setd(cruisers);
        }
        kout << c_number << TreatyName(type) << c_normal << " proposed";
        if (days > 0)
            kout << " for " << c_number << days << c_normal << " days";
        kout << ".\n";
    }
}

// 70d2:1139
static void BreakTreaty(Player *p, Empire *me)
{
    int32 chosen = 0;
    int32 allowed = EmpireMask();
    MaskClear(&allowed, me->id);
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (!Alive(&g_empires[i]))
            MaskClear(&allowed, i);
    if (ChooseEmpires(allowed, &chosen, String(""), RelationsList, 0) != 1)
        return;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (!MaskTest(chosen, i))
            continue;
        kout << Color(10) << (char)(i + 'A') << ": " << c_normal;
        Empire *o = &g_empires[i];
        Treaty *mine = &me->treaty[i];
        Treaty *theirs = &o->treaty[me->id];
        if (mine->type() == 0) {
            kout << "You have no relations with " << c_empire << Untr(o->name) << c_normal << "!\n";
        } else if (mine->a == 0) {
            kout << "You break the " << c_number << TreatyDesc(*mine) << c_normal << ".\n";
            SetTreaty(*mine, 0, 0);
            mine->setc(0);
            theirs->setc(4);
        } else if (theirs->c() == 1) {
            kout << "You have proposed a " << c_number << TreatyDesc(*theirs) << c_normal << " to "
                 << c_empire << Untr(o->name) << c_normal << ".  ";
            if (YesNo(0, "Do you wish to cancel the proposal")) {
                *theirs = *mine;
                mine->setc(0);
                theirs->setc(0);
            }
        } else if (theirs->c() == 4) {
            kout << "You have already sent word that you wish to break the treaty!\n";
        } else {
            theirs->setc(4);
            kout << "You send word that you wish to break the treaty.\n";
        }
    }
}

// 70d2:1787
static void ViewRelations(Empire *me)
{
    Relations(me, 0);
}

// 70d2:0000: the diplomacy menu.
void DiplomacyMenu(Player *p, Empire *me)
{
    if (!r_diplomacy.Bool())
        return;
    int16 r;
    do {
        Menu m("Diplomacy", "Diplomacy", 0xe, 2, 2, 0xe, 0xa, 0xd, 0xd, 7, 7, 7);
        m.prompt = new DiplomacyPrompt;
        if (r_neutral.Bool())
            m.Add(TreatyName(1), '1', 1);
        if (r_freetrade.Bool())
            m.Add(TreatyName(2), '2', 2);
        if (r_minor.Bool())
            m.Add(TreatyName(3), '3', 3);
        if (r_armed.Bool())
            m.Add(TreatyName(4), '4', 4);
        if (r_cruiser.Bool())
            m.Add(TreatyName(5), '5', 5);
        if (r_total.Bool())
            m.Add(TreatyName(6), '6', 6);
        if (r_break.Bool())
            m.Add("Break Treaty", '7', 7);
        m.Add("View Relations", '8', 8);
        if (r_custom.Bool())
            m.Add("Propose Other Treaty", '9', 9);
        r = m.Run(1, -1);
        switch (r) {
        case 1: case 2: case 3: case 4: case 5: case 6:
            Propose(p, me, r);
            break;
        case 7: BreakTreaty(p, me); break;
        case 8: ViewRelations(me); break;
        case 9: Propose(p, me, 7); break;
        }
    } while (r != -1);
}

// 70d2:1ddf: messages about treaties from the other empires.
void TreatyMessages(Player *p, Empire *me)
{
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *o = &g_empires[i];
        if (!Exists(*o))
            continue;
        Treaty *a = &me->treaty[o->id];
        Treaty *b = &o->treaty[me->id];
        if (a->c() == 4) {
            kout << Color(15) << Untr(o->name) << c_normal << " has ";
            if (a->a > 0)
                kout << "dishonorably ";
            kout << "broken a treaty with you!\n";
            SetTreaty(*a, 0, 0);
            a->setc(0);
            SetTreaty(*b, 0, 0);
            b->setc(0);
        }
        if (a->c() == 1) {
            if (me->id == g_galaxy.weaponsmaster && TotalPlanets(me) > 500 && a->type() != 2 &&
                a->type() != 1) {
                kout << "Because you are the Weapons Master, the treaty has been changed to Neutrality.\n";
                SetTreaty(*a, 1, a->a);
            }
            kout << Color(15) << Untr(o->name) << c_normal << " proposes a " << c_number
                 << TreatyName(a->type()) << c_normal;
            if (a->a > 0)
                kout << " for " << c_number << (int16)a->a << c_normal << " days";
            kout << ".\n";
            if (a->type() == 7) {
                kout << "It involves " << c_number << a->e() << c_normal << "% soldier defense and ";
                kout << c_number << a->d() << c_normal << "% cruiser defense.\n";
            }
            if (YesNo(1, "Do you accept")) {
                kout << "Treaty Accepted";
                *b = *a;
                b->setc(3);
                a->setc(0);
            } else {
                int16 type = a->type();
                uint8 days = a->a;
                kout << "Treaty Rejected";
                *a = *b;
                a->setc(0);
                SetTreaty(*b, type, days);
                b->setc(2);
            }
            kout << c_normal << ".\n";
        }
        if (a->c() == 3) {
            kout << c_number << Untr(o->name) << c_normal << " accepts the " << c_number
                 << TreatyDesc(*a) << c_normal << ".\n";
            a->setc(0);
            b->setc(0);
        }
        if (a->c() == 2) {
            kout << c_number << Untr(o->name) << c_normal << " rejects the " << c_number
                 << TreatyDesc(*a) << c_normal << " proposal.\n";
            *a = *b;
        }
    }
}
