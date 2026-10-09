// Double agents planted by other empires (segment 432f).
#include "sre.h"

// 432f:0000
struct DoubleAgentPrompt : MenuHook {
    Empire *e;
    explicit DoubleAgentPrompt(Empire *x) : e(x) {}
    int16 Call()
    {
        kout << "Your Intelligence Agency has determined that at least one covert agent ";
        kout << "is actually a double-spy, spying for another empire.\n";
        kout << "You have " << c_number << e->money.get() << c_normal << " credits.\n";
        return 1;
    }
};

static int16 Spies(Empire *o, Empire *e)
{
    return Exists(*o) && MaskTest(*OpMask(o, 8), e->id);
}

// 432f:006f
void DoubleAgents(Player *p, Empire *e)
{
    int32 n = e->agents;
    for (int16 i = 0; i < 25; i++)
        if (Spies(&g_empires[i], e))
            n--;
    while (n < 0) {
        int16 i = Random(25);
        if (Spies(&g_empires[i], e)) {
            MaskClear(OpMask(&g_empires[i], 8), e->id);
            n++;
        }
    }
    if (e->agents == n)
        return;
    if (Random(3) != 0)
        return;
    if (Random(3) == 0 && ShowScreen(0, "TwoTimer", 0))
        Pause();
    for (;;) {
        int32 cost[4];
        cost[1] = 430 * e->agents;
        cost[2] = e->agents + 100;
        cost[3] = 1037 * e->agents;
        Menu m("DoubleSpy", "ALERT!  Double Agents", 0xe, 5, 5, 0xe, 7, 0xf, 7, 7, 7, 7);
        m.prompt = new DoubleAgentPrompt(e);
        m.Add("Find a double-spy", '1', 1);
        m.Add("Eliminate all spies", '2', 2);
        m.Add("Bribe a double spy", '3', 3);
        m.Add("Ignore this threat", 'Q', 0x63);
        for (int16 i = 1; i <= 3; i++)
            m.Find(i)->AddColumn(cost[i]);
        if (g_system.regstate)
            m.Add("", 'V', 0x50);
        int16 r = m.Run(1, -1);
        if (r >= 1 && r <= 9 && cost[r] > e->money.get()) {
            kout << "You can not afford to do that!\n";
            continue;
        }
        switch (r) {
        case 1: {
            e->money.sub(cost[1]);
            n = 0;
            int32 found = 0;
            for (int16 i = 0; i < 25; i++) {
                Empire *o = &g_empires[i];
                if (Spies(o, e) && Random(3) != 0) {
                    MaskSet(&found, i);
                    MaskClear(OpMask(o, 8), e->id);
                    n++;
                }
            }
            for (int16 i = 0; i < 25; i++)
                if (MaskTest(found, i))
                    kout << "You found an agent from " << c_empire << Untr(g_empires[i].name)
                         << c_normal << ".\n";
            kout << "You found a total of " << Color(9) << n << c_normal;
            if (n == 1)
                kout << " double-spy.\n";
            else
                kout << " double-spies.\n";
            return;
        }
        case 2:
            if (!YesNo(0, "Are you sure"))
                continue;
            e->money.sub(cost[2]);
            // The count continues from the agents left after the
            // double-spies were taken out above.
            for (int16 i = 0; i < 25; i++) {
                if (Exists(g_empires[i])) {
                    MaskClear(OpMask(&g_empires[i], 8), e->id);
                    n++;
                }
            }
            e->agents = 0;
            kout << "You killed " << c_number << n << c_normal;
            if (n == 1)
                kout << "double-spy";
            else
                kout << "double-spies";
            kout << ".\n";
            return;
        case 3: {
            e->money.sub(cost[3]);
            int16 count = 0;
            int32 found = 0;
            for (int16 i = 0; i < 25; i++) {
                if (Spies(&g_empires[i], e) && Random(3) != 0) {
                    MaskSet(&found, i);
                    count++;
                }
            }
            int16 pick = Random(count);
            for (int16 i = 0; i < 25; i++) {
                if (!MaskTest(found, i))
                    continue;
                if (pick > 0) {
                    pick--;
                    continue;
                }
                Empire *o = &g_empires[i];
                MaskClear(OpMask(o, 8), e->id);
                MaskSet(OpMask(e, 8), o->id);
                kout << "You hired an agent from " << c_empire << Untr(o->name) << c_normal
                     << ".\n";
                break;
            }
            return;
        }
        case 0x50:
            Bank(p, e, &g_galaxy);
            continue;
        case 0x63:
            return;
        }
    }
}
