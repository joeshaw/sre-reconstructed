// The food market and feeding the people and army (segment 512f).
#include "sre.h"

void Bank(Player *p, Empire *e, Galaxy *g);            // 3e41:0000

static Resource r_foodmarket("FoodMarket.Name", "the Solar Food Market");     // 3bae

// 512f:0000: the price of food, given the supply.
static int32 FoodPrice(Galaxy *g, int32 extra)
{
    int32 q = g->d388 + extra;
    if (q < 0)
        q = 0;
    if (q < 160000)
        return 85 - q / 2000;
    return 5;
}

// 512f:0072
static const char *FoodDemand(Galaxy *g, int32 extra)
{
    int32 p = FoodPrice(g, extra);
    if (p < 10)
        return "very low";
    if (p < 20)
        return "low";
    if (p < 40)
        return "moderate";
    if (p < 60)
        return "medium";
    if (p < 80)
        return "high";
    return "great";
}

// 512f:0105: the food needed by the people (1), the army (2), or both.
static int32 FoodNeeded(Empire *e, int16 who)
{
    if (who == 1)
        return 38 * e->population / 100;
    if (who == 2)
        return 3 * e->soldiers / 100;
    return FoodNeeded(e, 1) + FoodNeeded(e, 2);
}

static void Key(char c)
{
    kout << c_bracket << '[' << c_bold << c << c_bracket << ']' << c_normal;
}

// 512f:018b
void FoodMarket(Player *p, Empire *e, Galaxy *g)
{
    const char *how = " credits per megaton, how many megatons of food would you like to ";
    const char *give = "How much will you give? ";
    const char *name = r_foodmarket.Str();
    kout << c_normal << "\nWelcome to " << Color(14) << name << '\n' << c_normal;
    int32 people = FoodNeeded(e, 1);
    int32 army = FoodNeeded(e, 2);
    int32 grown = e->planets[P_FOOD].count * e->planets[P_FOOD].b;
    int32 price = FoodPrice(g, e->food - people - army);
    const char *demand = FoodDemand(g, e->food - people - army);
    if (FoodNeeded(e, 0) > grown && e->food / 3 < FoodNeeded(e, 0))
        kout << Color(3)
             << "Your advisors inform you that your food planets do not produce enough food for you to be self-sufficient.\n"
             << c_normal;
    for (;;) {
        kout << "Current quantity available is " << c_number << g->d388 << c_normal << ".  ";
        kout << "Food is in " << demand << " demand.\n";
        kout << "You currently have " << c_number << e->food << c_normal << " megatons of food.\n";
        kout << "Would you like to ";
        Key('B');
        kout << "uy, ";
        Key('S');
        kout << "ell, ";
        if (g_system.regstate) {
            Key('V');
            kout << "isit the bank, ";
        }
        kout << "or ";
        Key('C');
        kout << "ontinue?  " << c_input;
        char c;
        do {
            c = GetChoice("CVBS", 1);
            if (!g_system.regstate && c == 'V')
                c = 0;
        } while (c == 0);
        if (c == 'B') {
            int32 max = e->money.get() / price;
            if (g->d388 < max)
                max = g->d388;
            int32 need = people + army - e->food;
            if (need < 0)
                need = 0;
            if (need > max)
                need = max;
            kout << "Buy" << c_normal << "\nAt " << c_number << price << c_normal << how << "buy? \n"
                 << c_bracket << '[' << c_bold << "MAX=" << max << c_bracket << ']' << c_normal
                 << "  ";
            int32 n;
            kin >> defval(need > 0 ? need : 0) >> maxval(max) >> n;
            e->money.sub(n * price);
            if (g->d3ac < 1500000000)
                g->d3ac += n * price / 1000;
            e->food += n;
            g->d388 -= n;
        } else if (c == 'S') {
            kout << "Sell" << c_normal << "\nAt " << c_number << price / 2 << c_normal << how
                 << "sell? \n" << c_bracket << '[' << c_bold << "MAX=" << e->food << c_bracket << ']'
                 << c_normal << "  ";
            int32 n;
            kin >> defval(0) >> maxval(e->food) >> n;
            e->money.add(n * (price / 2));
            g->d3ac -= n * (price / 2) / 1000;
            if (g->d3ac < 0)
                g->d3ac = 0;
            e->food -= n;
            g->d388 += n;
        } else if (c == 'V') {
            kout << "Visit Bank" << c_normal << '\n';
            Bank(p, e, g);
        }
        if (c != 'C')
            continue;
        kout << "Continue" << c_normal << "\n\n";
        int32 food = e->food;
        uint8 flags = e->b1c4;
        int8 b1c5 = e->b1c5;
        int32 starving;
        memcpy(&starving, e->u_1c7 + 0x5c, 4);
        int32 fedpeople, fedarmy;
        if (people > 0) {
            int32 def = people < food ? people : food;
            if ((p->prefs1 >> 3) & 1)
                kout << "People:  " << c_number << people << c_normal << '\n';
            else
                kout << "Your people require " << c_number << people << c_normal
                     << " megatons of food.\n";
            kout << give;
            kin >> defval(def) >> maxval(food) >> fedpeople;
            if (fedpeople < people) {
                flags |= 2;
                starving = (people - fedpeople) * e->population / people;
                if (starving < 0)
                    starving = 0;
            }
            food -= fedpeople;
        } else {
            fedpeople = 0;
        }
        if (army > 0) {
            int32 def = army < food ? army : food;
            if ((p->prefs1 >> 3) & 1)
                kout << "Army:    " << c_number << army << c_normal << '\n';
            else
                kout << "\nYour Army needs " << c_number << army << c_normal
                     << " megatons of food.\n";
            kout << give;
            kin >> defval(def) >> maxval(food) >> fedarmy;
            if (fedarmy < army) {
                int16 k = (int16)(-((39 * (army - fedarmy)) / army));
                if (b1c5 > k)
                    b1c5 = (int8)k;
                flags |= 8;
            }
            food -= fedarmy;
        } else {
            fedarmy = 0;
        }
        if (fedpeople < people || fedarmy < army) {
            kout << "\nFailure to feed your people/army will lead to civil war and/or loss of forces.\n";
            if (YesNo(1, "Do you wish to reconsider your actions")) {
                kout << '\n';
                continue;
            }
        }
        e->b1c4 = flags;
        e->b1c5 = b1c5;
        memcpy(e->u_1c7 + 0x5c, &starving, 4);
        e->food = food;
        return;
    }
}
