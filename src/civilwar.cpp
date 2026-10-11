// Civil wars from neglected planets or starvation, and desertion from an
// unpaid or unfed army (segment 44e2).
#include "sre.h"
#include "itemlist.h"


// 44e2:04f2: lose pct percent of the military.
static void LoseMilitary(int32 pct, Empire *e, ItemList *l)
{
    int32 soldiers = pct * e->soldiers / 100;
    int32 fighters = pct * e->fighters / 100;
    int32 stations = pct * e->stations / 100;
    int32 heavy = pct * e->heavycruisers / 100;
    int32 generals = pct * e->generals / 100;
    int32 carriers = pct * e->carriers / 100;
    int32 light = pct * e->lightcruisers / 100;
    e->soldiers -= soldiers;
    e->fighters -= fighters;
    e->stations -= stations;
    e->heavycruisers -= heavy;
    e->generals -= generals;
    e->carriers -= carriers;
    e->lightcruisers -= light;
    l->SkipZero();
    l->Add(soldiers, "soldier");
    l->Add(fighters, "fighter");
    l->Add(stations, "defense station");
    l->Add(light, "light cruiser");
    l->Add(heavy, "heavy cruiser");
    l->Add(generals, "general");
    l->Add(carriers, "carrier");
}

// 44e2:07cc
void CivilWar(Player *p, Empire *e)
{
    (void)p;
    int16 civil = 0;
    int32 pct = 0;
    const char *cause = 0, *size = 0;
    int32 killed = 0;
    int32 total = 0;
    ItemList military(c_normal, c_number);
    ItemList lost(c_normal, c_number);
    lost.SkipZero();
    if (e->shortfalls & SF_NEGLECT) {
        // Neglected planets revolt.
        InsurgencyUp(e);
        for (int16 i = 0; i < 10; i++) {
            int32 n = e->neglected[i];
            if (Planets(e, i)->count < n)
                n = Planets(e, i)->count;
            lost.Add(n, PlanetName((int8)i), "");
            g_galaxy.marketplanets += n;
            total += n;
            Planets(e, i)->count -= n;
        }
        // The original divides by zero if no planets are left.
        int32 tp = TotalPlanets(e);
        pct = tp ? 100 * total / tp : 100;
        killed = pct * e->population / 100;
        e->population -= killed;
        LoseMilitary(pct, e, &military);
        if (lost.count > 0) {
            if (pct < 20)
                size = "minor";
            else if (pct < 30)
                size = "small";
            else if (pct < 50)
                size = "large";
            else if (pct < 70)
                size = "major";
            else
                size = "disastrous";
            cause = "money";
            civil = 1;
        } else {
            cause = "unemployment";
        }
    } else if (e->shortfalls & SF_STARVED) {
        // Starvation.
        int32 starving = e->starving;
        // The original divides by zero if the population is gone.
        pct = e->population ? 100 * starving / e->population : 0;
        if (pct < 0)
            pct = 0;
        if (pct > 30)
            pct = 30;
        killed = pct * e->population / 100;
        e->population -= killed;
        InsurgencyUp(e);
        for (int16 i = 0; i < 10; i++) {
            int32 n = pct * Planets(e, i)->count / 100;
            lost.Add(n, PlanetName((int8)i), "");
            g_galaxy.marketplanets += n;
            total += n;
            Planets(e, i)->count -= n;
        }
        LoseMilitary(pct, e, &military);
        if (lost.count > 0) {
            cause = "food";
            if (pct < 10)
                size = "inconsequential";
            else if (pct < 20)
                size = "minor";
            else
                size = "major";
            civil = 1;
        } else {
            cause = "food shortages";
        }
    } else {
        return;
    }
    if (!civil && killed > 0) {
        kout << c_normal << "Your people are in riots due to " << cause
             << ", but your officials prevent a civil war from breaking out.";
        if (killed > 0)
            kout << "  However, " << c_number << killed << c_normal
                 << " million people were killed in the riots.";
        kout << '\n';
    }
    if (civil) {
        if (Random(100) <= pct && ShowScreen(0, "CivilWar", 0))
            Pause();
        kout << "Without " << cause << ", your empire plunges into a " << size << Color(13)
             << " CIVIL WAR" << c_normal << "!\n" << c_normal << "You lose " << lost
             << " planets (total " << Color(14) << total << c_normal << ')';
        if (killed > 0)
            kout << " and " << c_number << killed << c_normal << " million people";
        kout << ".\n";
        if (r_civilwar.Bool() && pct > 30)
            g_log2 << c_empire << Untr(e->name) << c_normal << " has suffered a major civil war!\n";
    }
    if (military.count > 0)
        kout << c_normal << "You also lose " << military << ".\n";
    HLine(Color(8), 0x4b);
    if (!civil)
        return;
    const char *topic = "civil-war";
    if (YesNo(pct > 25, "Display hints to avoid civil wars")) {
        kout << '\n';
        ShowHelpTopic(topic, "general");
        if (e->planets[P_TOURISM].count > e->planets[P_ORE].count) {
            ShowHelpTopic(topic, "tourism");
            if (e->planets[P_ANTIPOLLUTION].output * e->planets[P_ANTIPOLLUTION].count <
                e->planets[P_PETROLEUM].count)
                kout << "Also, tourism profits decrease when you have pollution caused by petroleum planets.";
            kout << "\n\n";
        }
        if (e->planets[P_FOOD].output * e->planets[P_FOOD].count > FoodNeeded(e, 0))
            ShowHelpTopic(topic, "long-term-food");
        ShowHelpTopic(topic, "continuous");
        if (g_system.regstate)
            ShowHelpTopic(topic, "loan");
        kout << "\n\nHope this helps!\n\n";
    }
    if (e->favor >= 25 && pct > 25 && e->money.get() + e->bank.get() < 100000000 &&
        (e->soldiers + e->fighters + e->stations + e->heavycruisers + e->agents < 2000 ||
         TotalPlanets(e) < 300)) {
        kout << "The Galactic Coordinator offers to give you some funds to recover from the civil war.  ";
        if (YesNo(1, "Do you accept")) {
            int32 x = e->favor;
            e->favor -= 25;
            if (x > 100)
                x = 100;
            x = x * (15 * TotalPlanets(e));
            e->money.add(x);
            kout << "You receive " << c_number << x << c_normal << " credits.\n";
        } else {
            kout << "The Galactic Coordinator wishes you luck in rebuilding your empire!\n";
        }
    }
    HLine(Color(8), 0x4b);
}

// 44e2:0000
void Desertion(Player *p, Empire *e)
{
    (void)p;
    if (e->effnext >= 0) {
        e->effectiveness = (int8)(e->effectiveness + e->effchange);
        if (e->effectiveness < 0)
            e->effectiveness = 0;
        if (e->effectiveness > 100)
            e->effectiveness = 100;
        return;
    }
    e->effectiveness = 0;
    if (!(e->shortfalls & SF_ARMY))
        return;
    int16 k = (int16)-e->effnext;
    if (k > 20)
        k = 20;
    if (k < 0)
        k = 0;
    int32 soldiers = e->soldiers * k / 100;
    int32 fighters = e->fighters * k / 100;
    int32 stations = e->stations * k / 100;
    int32 heavy = e->heavycruisers * k / 100;
    int32 generals = e->generals * k / 100;
    int32 carriers = e->carriers * k / 100;
    int32 light = e->lightcruisers * k / 100;
    e->soldiers -= soldiers;
    e->fighters -= fighters;
    e->stations -= stations;
    e->heavycruisers -= heavy;
    e->generals -= generals;
    e->carriers -= carriers;
    e->lightcruisers -= light;
    if (soldiers > 0)
        kout << c_number << soldiers << c_normal << " troops desert you.\n";
    if (generals > 0)
        kout << c_number << generals << c_normal << " generals leave your army.\n";
    if (fighters > 0)
        kout << c_number << fighters << c_normal
             << " fighters decay due to lack of maintainance funds.\n";
    if (heavy > 0)
        kout << c_number << heavy << c_normal << " heavy cruisers decompose due to lack of funds.\n";
    if (light > 0)
        kout << c_number << light << c_normal
             << " light cruisers fail to work due to lack of proper maintenance.\n";
    if (stations > 0)
        kout << c_number << stations << c_normal << " defense stations fall into ruins.\n";
    if (carriers > 0)
        kout << c_number << carriers << c_normal << " carriers are no longer functional.\n";
    if (soldiers > 0 || generals > 0 || fighters > 0 || heavy > 0 || stations > 0 ||
        carriers > 0 || light > 0)
        HLine(Color(8), 0x4b);
}
