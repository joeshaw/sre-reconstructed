// End of a turn: donations and deposits, and population change (segment 5f38).
#include "sre.h"

// 5f38:0000
void Population(Player *p, Empire *e)
{
    int16 n = 0;
    int32 tp = TotalPlanets(e);
    int32 born = (tp / 2 + 101 * e->planets[P_URBAN].count + e->population) / (Random(10) + 23) + 1;
    int32 deaths = e->population * (e->population / (Random(10) + 17)) / (TotalPlanets(e) + 1) / 10;
    int32 emigrants = e->population * ((int32)(int16)(e->insurgency * e->insurgency) + 1) / 80;
    int32 immigrants = (36 * (e->planets[P_EDUCATION].output * e->planets[P_EDUCATION].count) +
                        e->population) /
                       ((int32)(int16)(e->insurgency * e->insurgency) + 50);
    int32 untaxed = 100 - (int32)e->taxrate;
    immigrants = untaxed * (immigrants * untaxed) / 10000;
    emigrants += emigrants * ((int32)e->taxrate * (int32)e->taxrate + 1) / 200;
    emigrants += e->planets[P_TOURISM].count * TotalPlanets(e) / 100000;
    emigrants += e->pollution / 100;
    deaths += e->planets[P_ORE].count / 10;
    int32 pollution = 10 * e->pollution / (TotalPlanets(e) + 1);
    if (pollution < 0)
        pollution = 0;
    deaths += pollution;
    if (emigrants < 0)
        emigrants = 0;
    if (e->population > 2000000000) {
        immigrants = 0;
        born = 0;
    }
    while (emigrants + deaths > e->population && n < 100) {
        emigrants -= emigrants / 7;
        deaths -= deaths / 7;
        n++;
    }
    while (emigrants + deaths > e->population)
        emigrants--;
    if (e->population < deaths)
        deaths = e->population;
    if (born < 0)
        born = 0;
    if (deaths < 0)
        deaths = 0;
    if (immigrants < 0)
        immigrants = 0;
    if (emigrants < 0)
        emigrants = 0;
    if (deaths < pollution)
        pollution = deaths;
    e->population += born + immigrants - emigrants - deaths;
    if (e->population < 0)
        e->population = 0;
    HLine(Color(8), 0x4b);
    if (Random(10) == 0 && !(e->shortfalls & SF_NEGLECT) && !(e->shortfalls & SF_STARVED) &&
        !(e->shortfalls & SF_ARMY) && !(e->shortfalls & SF_UNUSED) && e->pollution == 0 &&
        e->favor > 0 && e->insurgency == 0)
        kout << "Your people love thee and hope you rule forever!\n\n";
    if (born > 0)
        kout << c_number << born << c_normal << " million people were born in your empire.\n";
    if (deaths > 0) {
        kout << c_number << deaths << c_normal << " million citizens die";
        if (pollution > 0)
            kout << ", including " << c_number << pollution << c_normal
                 << " million from galactic pollution";
        kout << ".\n";
    }
    if (immigrants > 0)
        kout << c_number << immigrants << c_normal << " million people immigrate to your empire.\n";
    if (emigrants > 0)
        kout << c_number << emigrants << c_normal
             << " million citizens emigrate from your empire.\n";
    kout << "A net population change of " << c_number;
    int32 net = born - deaths + immigrants - emigrants;
    if (net > 0)
        kout << '+';
    kout << net << c_normal << " million people.\n";
    HLine(Color(8), 0x4b);
}

// 5f38:07ae
void EndOfTurn(Player *p, Empire *e, Galaxy *g)
{
    kout << '\n';
    if (e->turnsused == e->turnsperday - 1)
        kout << "This is your last turn today.\n";
    int32 money = e->money.get();
    int16 donate = HasPref(p->prefs1, PF1_DONATE);
    int32 gift = money;
    if (1000 * e->income.get() / 100 < gift)
        gift = 1000 * e->income.get() / 100;
    if ((HasPref(p->prefs1, PF1_ASKDONATE)) && money > 0) {
        donate = YesNo(donate, "Donate to the Galactic Coordinator");
        if (donate) {
            kout << "Donate how much?  ";
            kin >> maxval(money) >> defval(gift) >> gift;
        }
    }
    if (donate)
        money -= gift;
    if (g_system.regstate) {
        int16 deposit = HasPref(p->prefs1, PF1_DEPOSIT);
        if ((HasPref(p->prefs1, PF1_ASKDEPOSIT)) && money > 0)
            deposit = YesNo(deposit, "Deposit the rest of your funds into the bank");
        if (deposit) {
            int32 old = e->bank.get();
            e->bank.add(money);
            money -= e->bank.get() - old;
        }
    }
    if (donate) {
        g->coordfund += gift / 1000;
        // The original divides by zero (and aborts) when there was no income.
        int32 k = e->income.get() ? gift / e->income.get() / 10 : 0;
        if (k > 10)
            k = 10;
        if (k < 0)
            k = 0;
        e->favor += k;
    }
    e->money.set(money);
}
