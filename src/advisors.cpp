// The advisors' reports (segment 3ab7).
#include "sre.h"

static Color c_advisor(10);     // DS:0094 (3ab7:0c22)

static void Advisor(const char *title)
{
    kout << "Your " << c_advisor << title << c_normal;
}

// 3ab7:0000
void Advisors(Player *p, Empire *e)
{
    (void)p;
    const char *hint = "hint";
    kout << '\n';
    int32 need = FoodNeeded(e, 0);
    int32 grown = e->planets[P_FOOD].count * e->planets[P_FOOD].output;
    int32 food = e->food;
    Advisor("Agricultural Advisor");
    kout << " reports that your food planets produce " << c_number << grown << c_normal
         << " megatons of food, and your empire requires " << c_number << need << c_normal
         << " megatons.";
    int16 years = 0;
    while (food > 0 && years < 30) {
        years++;
        food += grown - need;
    }
    if (years > 0 && years < 30)
        kout << "  Your food will last for " << c_number << years << c_normal << " years.";
    kout << "\n\n";
    if (Random(10) == 5) {
        Advisor("Economic Advisor");
        kout << " is not in her office.  Her employees say that she is out playing tennis, and she will be back shortly.\n\n";
    } else {
        Advisor("Economic Advisor");
        kout << "'s report states that your empire has an efficiency of " << c_number << e->efficiency
             << c_normal << " credits per planet (profit).";
        if (e->efficiency > 2000)
            kout << "  Congratulations!";
        else if (e->efficiency > 1500)
            kout << "  That is a respectable amount of profit.  You are a wise ruler!";
        else if (e->efficiency < 0)
            kout << "  This is not very good for your empire!!!";
        int32 money = e->planets[P_TOURISM].count + e->planets[P_PETROLEUM].count +
                      e->planets[P_ORE].count + e->planets[P_URBAN].count +
                      e->planets[P_EDUCATION].count / 2;
        if (2 * money < TotalPlanets(e))
            kout << "  In her opinion, you have too few money-producing planets.";
        int16 r1 = Random(10);
        int16 r2 = Random(10);
        if (r1 + r2 + 30 < e->taxrate)
            kout << "  She thinks that you tax your people too much.";
        r1 = Random(5);
        r2 = Random(5);
        if (r1 + r2 > e->taxrate)
            kout << "  She thinks that you should tax your people to raise revenue for the empire.";
        kout << "\n\n";
    }
    if (e->year < 0) {
        // Advice for empires still under protection.
        const char *small = "hint-small-empire";
        if (TotalPlanets(e) < 100)
            ShowHelpTopic(small, "no-planets");
        else if (e->year > -5)
            ShowHelpTopic(small, "almost-ready");
        else if (TotalPlanets(e) / 2 > e->planets[P_TOURISM].count + e->planets[P_ORE].count +
                                           e->planets[P_PETROLEUM].count +
                                           e->planets[P_URBAN].count)
            ShowHelpTopic(small, "no-money");
        else if (e->cmdship == 0)
            ShowHelpTopic(small, "command-ship");
        else if (e->agents == 0)
            ShowHelpTopic(small, "covert-agents");
        return;
    }
    int16 sick = e->year % 10 == 3;
    if (!sick && (e->soldiers + 49) / 50 > e->generals) {
        Advisor("Chief of Military Maneuvers");
        kout << " asks you to buy more generals to command your forces.  You have " << c_number
             << e->soldiers << c_normal << " soldiers and your generals can lead " << c_number
             << 50 * e->generals << c_normal << " to battle.\n\n";
    }
    if (!sick && e->carriers + HasWeapon(e, W_MOTHERSHIP) < (e->fighters + 99) / 100) {
        Advisor("Chief of Military Maneuvers");
        kout << " asks you to buy more carriers to carry your fighters.  You have " << c_number
             << e->fighters << c_normal << " fighters and your carriers can take " << c_number
             << 100 * e->carriers << c_normal << " to battle.\n\n";
    }
    if (sick) {
        Advisor("Chief of Military Maneuvers");
        kout << " asks you if you have any asprin.  He seems to be ill.\n\n";
    }
    if (e->agents > 0) {
        int32 tp = TotalPlanets(e);
        int32 a = (e->agents + tp / 2) / (tp + 1);
        int32 b = (tp + e->agents / 2) / (e->agents + 1);
        if (a < 1)
            a = 1;
        if (b < 1)
            b = 1;
        Advisor("Chief of Intelligence");
        kout << " reports that your covert agent / planet ratio is " << c_number << a << c_normal
             << ':' << c_number << b << c_normal << ".\n\n";
    }
    if (e->favor > 100 || e->favor < -50) {
        const char *un = e->favor < 0 ? "un" : "";
        Advisor("Chief of Relations");
        kout << " says that the Galactic Coordinator looks " << un << "favorably upon you.\n\n";
    }
    if (e->insurgency > 0)
        ShowHelpTopic(hint, "insurgency");
    if (e->taxrate == 23)
        ShowHelpTopic(hint, "tax-rate");
    if (e->soldiers == 0)
        ShowHelpTopic(hint, "soldiers");
    if (e->stations == 0)
        ShowHelpTopic(hint, "defense-stations");
    if (e->pollution != 0)
        ShowHelpTopic(hint, "pollution");
    if (e->tickets == 0)
        ShowHelpTopic(hint, "lottery");
    if (e->researchpoints == 0)
        ShowHelpTopic(hint, "research");
    Pause();
}
