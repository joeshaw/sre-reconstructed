// Battle reports and attack news (segment 3f62).
#include "sre.h"
#include "battle.h"

const char *BATTLE_FILE = "Data/Battles";       // DS:0d5c

// 3f62:00cb: what the victim of an attack is told.
void AttackMessage(OStream &o, const char *attacker, int16 severity)
{
    if (severity > 0)
        o << Color(14);
    else
        o << c_empire;
    o << attacker << c_normal << ' ';
    if (severity > 10)
        o << "demolished your empire and spread terror throughout the land!";
    else if (severity > 4)
        o << "invaded your empire and caused devastation!";
    else if (severity > 2)
        o << "caused considerable damage to your land in a fierce attack.";
    else if (severity > 0)
        o << "attacked you and defeated your forces.";
    else if (severity > -3)
        o << "attacked but was repelled by your forces.";
    else if (severity > -6)
        o << "attempted an invasion but was unable to penetrate your defenses.";
    else
        o << "made a feeble attempt to invade your lands.";
}

// 3f62:01c1
int16 ShowBattleReports(Player *p, Empire *e)
{
    return ShowReports(e->id, BATTLE_FILE, (p->prefs2 >> 3) & 1, 4, 12);
}

// 3f62:01f5
void ClearBattleReports()
{
    ClearStreamFile(BATTLE_FILE);
}

// 3f62:0209: the news of an attack.
void AttackNews(Empire *a, Empire *d, int16 success, int16 severity)
{
    if (success) {
        g_log2 << c_empire << String(a->name) << c_normal;
        if (Exists(*d) && TotalPlanets(d) > 0 && d->population > 0) {
            if (severity > 10)
                g_log2 << " pulverized";
            else if (severity > 8)
                g_log2 << " devastated";
            else if (severity > 4)
                g_log2 << " demolished";
            else if (severity > 2)
                g_log2 << " invaded";
            else
                g_log2 << " defeated";
        } else {
            g_log2 << " completely destroyed";
        }
        if (!r_attackvictim.Bool())
            g_log2 << " a";
    } else {
        g_log2 << c_empire << String(a->name) << c_normal << " unsuccessfully attacked";
        if (!r_attackvictim.Bool())
            g_log2 << " a";
    }
    if (r_attackvictim.Bool()) {
        g_log2 << ' ';
        if (success)
            g_log2 << Color(9);
        else
            g_log2 << Color(11);
        g_log2 << String(d->name) << c_normal << ".\n";
        return;
    }
    if (TotalPlanets(a) > TotalPlanets(d) && NetWorth(a) > NetWorth(d))
        g_log2 << " smaller";
    else if (TotalPlanets(a) < TotalPlanets(d) && NetWorth(a) < NetWorth(d))
        g_log2 << " larger";
    else
        g_log2 << "nother";
    g_log2 << " empire.\n";
}
