// Random events at the start of a turn: pirates, revolts, disasters,
// research developments and weapons offers (segment 6572).
#include "sre.h"
#include "research.h"

static Resource r_nuclear("Attacks.Nuclear", (int32)1);    // 816a
static Resource r_chemical("Attacks.Chemical", (int32)1);  // 8176
static Resource r_psionic("Attacks.Psionic", (int32)1);    // 8182

static inline PlanetSet &P(Empire *e, int16 type) { return e->planets[type]; }

// 6572:0000: good news.
static void Good()
{
    kout << Color(2) << "\xfe " << c_normal;
}

// 6572:0040: bad news.
static void Bad()
{
    kout << Color(1) << "\xfe " << c_normal;
}

// 6572:0080
static void Appeased(Empire *e, Pirate *p)
{
    if (p->hatred[e->id] > 0)
        p->hatred[e->id]--;
}

static void MaybeScreen(int16 odds, const char *name)
{
    if (Random(odds) == 0 && ShowScreen(0, name, 0))
        Pause();
}

static void AddClamp(int32 &v, int32 n)
{
    v += n;
    if (v < 0)
        v = 0;
}

// 6572:00ac: what happens to an empire because of the pirates and
// fate.
static void Fate(Player *pl, Empire *e, Pirate *p)
{
    int16 lost[10];
    if (e->turnsused == g_galaxy.grantturns / 2 && e->year > 0 &&
        (e->id == g_galaxy.planetmaster || TotalPlanets(e) / 100 > PiratePlanets(p))) {
        int32 total = 0;
        MaybeScreen(5, "Revolt");
        for (int16 i = 0; i < 10; i++) {
            int32 count = Planets(e, i)->count;
            if (count > 300)
                count = 300;
            uint16 m = (uint16)((count * (int32)(e->taxrate / 10)) / 100);
            if (m > 20)
                m = 20;
            lost[i] = (int16)(((int32)(int16)m * b_rand()) / 0x8000);
            if ((int32)lost[i] >= count)
                lost[i] = (int16)(count - 1);
            if (lost[i] < 0)
                lost[i] = 0;
            total += lost[i];
        }
        if (total > 0) {
            for (int16 i = 0; i < 10; i++)
                Planets(e, i)->count -= lost[i];
            Bad();
            kout << c_number << total << c_normal << " planets revolt under the leadership of ";
            switch (Random(40) - 1) {
            case 0: kout << Color(11) << "Amit Patel"; break;
            case 1: kout << Color(13) << "Emperor Reko"; break;
            case 2: kout << Color(14) << "Bill Clinton"; break;
            case 3: kout << Color(9) << "Gengher Kahn"; break;
            case 4: kout << Color(2) << "King Jello"; break;
            case 5: kout << Color(3) << "a Talking Horse"; break;
            case 6: kout << Color(6) << "Ziggy the Dinosaur"; break;
            case 7:
                if (g_remote) {
                    kout << Color(10) << String(g_system.regto);
                    break;
                }
                // fall through
            default:
                PutPirate(kout, p);
                for (int16 i = 0; i < 10; i++)
                    p->planets[i] += lost[i];
                Appeased(e, p);
                break;
            }
            kout << c_normal << ".\n";
        }
    }
    if (NetWorth(e) < PirateNetWorth(p) / 20)
        return;

    if (Random(5) == 0 && TotalPlanets(e) > 60 && e->year > 0) {
        int32 n = (e->money.get() / 10000) * Random(100);
        if (n > 0) {
            AddClamp(p->money, n);
            e->money.sub(n);
            MaybeScreen(15, "Thieves");
            Bad();
            kout << c_number << n << c_normal << " credits were stolen by ";
            PutPirate(kout, p) << c_normal << "!\n";
        }
    }
    if (Random(45) == 0 && NetWorth(e) > 10000 && e->year > 0) {
        int32 n = (e->heavycruisers / 100) * Random(10);
        if (n > 0 && e->heavycruisers / 5 >= n) {
            e->heavycruisers -= n;
            AddClamp(p->heavycruisers, n);
            Bad();
            kout << c_number << n << c_normal << " cruisers were stolen by ";
            PutPirate(kout, p) << c_normal << ".\n";
        }
    }
    if (Random(45) == 0 && NetWorth(e) < 100000) {
        int16 k = (int16)(10 * TotalPlanets(e));
        int32 n = (int16)(((int32)k * b_rand()) / 0x8000);
        if (n > 0) {
            e->heavycruisers += n;
            Good();
            kout << c_number << n << c_normal << " cruisers were given to you by Aunt Edna!\n";
        }
    }
    if ((int32)e->taxrate * e->taxrate > Random(8000) && e->population > 500 &&
        TotalPlanets(e) > 0) {
        int16 k = e->taxrate;
        int32 n = (e->population / 200) * (((int32)k * b_rand()) / 0x8000);
        int32 r = 3 * e->score / TotalPlanets(e);
        if (r > 100)
            n /= 2;
        if (r > 50)
            n -= n / 5;
        if (r > 10)
            n -= n / 11;
        if (r > 1)
            n -= n / 17;
        if (n > 0) {
            int16 violence = 0;
            e->population -= n;
            if (e->insurgency < 7) {
                violence = 1;
                InsurgencyUp(e);
                if (Random(3) == 0)
                    InsurgencyUp(e);
                if (Random(7) == 0)
                    InsurgencyUp(e);
            }
            MaybeScreen(15, "HighTax");
            Bad();
            kout << c_number << n << c_normal
                 << " million people were killed in riots due to high tax rates!\n";
            if (violence) {
                Bad();
                kout << "Violence increases in your empire!\n";
            }
        }
    }
    if (Random(30) == 0 && TotalPlanets(e) > 30 && e->year > 0) {
        int32 n = (e->food / 700) * Random(100);
        if (n > 0) {
            e->food -= n;
            Bad();
            int16 who = (int16)(Random(10) - 1);
            if (who == 4 && !g_remote)
                who = -1;
            switch (who) {
            case 0:
                kout << c_number << n << c_normal << " megatons of food were eaten by " << Color(6)
                     << "velociraptors" << c_normal << "!\n";
                break;
            case 1:
                kout << Color(6) << "Crop disease" << c_normal << " destroyed " << c_number << n
                     << c_normal << " megatons of food!\n";
                break;
            case 2:
                kout << Color(10) << "Arcadian locusts" << c_normal << " destroy " << c_number << n
                     << c_normal << " megatons of food!\n";
                break;
            case 3:
                kout << Color(3) << "Amit Patel" << c_normal << " swallowed " << c_number << n
                     << c_normal << " megatons of food in one gulp!\n";
                break;
            case 4:
                kout << Color(15) << "Your hungry Sysop" << ", " << String(g_system.sysop) << ','
                     << c_normal << " ate " << c_number << n << c_normal << " megatons of food!\n";
                break;
            default:
                AddClamp(p->food, n);
                Appeased(e, p);
                if (Random(15) == 0 && ShowScreen(0, "FoodLost", 0)) {
                    Pause();
                    Bad();
                }
                kout << c_number << n << c_normal << " megatons of food were stolen by ";
                PutPirate(kout, p) << c_normal << "!\n";
                break;
            }
        }
    }
    if (Random(1000) <= 50 && P(e, P_FOOD).count > 100 && e->year > 0) {
        int32 n = P(e, P_FOOD).count * (Random(5000) / 3);
        if (n > 0 && e->food >= n) {
            e->food -= n;
            Bad();
            kout << c_number << n << c_normal
                 << " megatons of food were destroyed due to harsh weather.\n";
        }
    }
    if (Random(1000) <= 30) {
        int32 n = Random(5000);
        if (n > 0) {
            e->food += n;
            Good();
            kout << "You received " << c_number << n << c_normal << " megatons of ";
            switch (Random(5)) {
            case 0: kout << "potatoes and onions"; break;
            case 1: kout << "carrots and peas"; break;
            case 2: kout << "leafy vegetables"; break;
            case 3: kout << "candy bars"; break;
            case 4: kout << "cheese"; break;
            }
            kout << " for your birthday.\n";
        }
    }
    if (Random(1000) <= 30 && P(e, P_PETROLEUM).count > 10 && e->year > 0) {
        int16 k = (int16)P(e, P_PETROLEUM).count;
        int32 n = (int16)((((int32)k * b_rand()) / 0x8000) / 3);
        if (n > 0) {
            e->pollution += n;
            Bad();
            kout << c_number << n << c_normal
                 << " exatons of toxic fumes were released due to a pumping accident.\n";
        }
    }
    if (Random(1000) <= 10 && P(e, P_PETROLEUM).count > 100 && e->year > 0) {
        int16 k = (int16)P(e, P_PETROLEUM).count;
        int32 n = (int16)((((int32)k * b_rand()) / 0x8000) / 10);
        if (n > 0) {
            e->pollution += n;
            Bad();
            kout << c_number << n << c_normal
                 << " exatons of toxic fumes were released when one of your advisors burped.\n";
        }
    }
    if (Random(1000) <= 50 && P(e, P_SUPPLY).count > 5 && e->year > 0) {
        int32 n = (P(e, P_SUPPLY).count / 3) * Random(25);
        if (n > 0 && e->soldiers >= n) {
            e->soldiers -= n;
            AddClamp(p->soldiers, n);
            Bad();
            kout << c_number << n << c_normal << " soldiers desert your training camps.\n";
        }
    }
    if (Random(1000) <= 10) {
        int32 n = 5 * TotalPlanets(e);
        if (n > 0) {
            e->soldiers += n;
            Good();
            kout << c_number << n << c_normal << " soldiers joined your army.\n";
        }
    }
    if (Random(1000) <= 50 && e->generals > 50 && e->year > 0) {
        AddClamp(p->generals, 1);
        e->generals -= 1;
        Bad();
        kout << c_number << (int16)1 << c_normal << " general was captured by ";
        PutPirate(kout, p) << c_normal << ".\n";
    }
    if (Random(1000) <= 10 && e->generals > 50 && e->year > 0) {
        e->generals -= 1;
        Bad();
        kout << c_number << (int16)1 << c_normal
             << " general lost a game of Solar Hangman, and was hanged.\n";
    }
    if (Random(1000) <= 10 && e->generals > 50 && e->year > 0) {
        e->generals -= 1;
        Bad();
        kout << c_number << (int16)1 << c_normal
             << " general was lost bungee-jumping off the cliffs of Raekel IV.\n";
    }
    if (Random(1000) <= 30 && e->agents > 500 && e->year > 0) {
        int16 k = e->agents > 3000 ? (int16)200 : (int16)(e->agents / 20);
        uint16 n = (uint16)(((int32)k * b_rand()) / 0x8000);
        if (n > 0) {
            AddClamp(p->agents, (int32)n);
            e->agents -= (int32)n;
            Bad();
            kout << c_number << n << c_normal << " covert agents were captured by ";
            PutPirate(kout, p) << c_normal << ".\n";
            Appeased(e, p);
        }
    }
    if (Random(1000) <= 50 && e->soldiers > 1000 && e->year > 0) {
        int16 k = (int16)(e->soldiers / 20);
        int32 n = (int16)(((int32)k * b_rand()) / 0x8000);
        if (n > 0) {
            AddClamp(p->soldiers, n);
            e->soldiers -= n;
            Bad();
            kout << c_number << n << c_normal << " soldiers have been forced to join the ";
            PutPirate(kout, p) << c_normal << " force.\n";
            Appeased(e, p);
        }
    }
    if (Random(90) == 0 && TotalPlanets(e) > 90 && e->year > 0) {
        int32 total = 0;
        for (int16 i = 0; i < 10; i++) {
            int16 k = (int16)(Planets(e, i)->count / 179);
            int16 x = (int16)(((int32)k * b_rand()) / 0x8000);
            if (x < 0)
                x = 0;
            if ((int32)x > Planets(e, i)->count / 100) {
                g_errlog << "random() - planets lost @14:  " << x << " > "
                         << Planets(e, i)->count << " [" << i << "]\n";
                x = 0;
            }
            total += x;
            Planets(e, i)->count -= x;
            lost[i] = x;
        }
        if (total > 0) {
            MaybeScreen(15, "Planets");
            Bad();
            kout << c_number << total << c_normal << " planet" << (total == 1 ? " was " : "s were ");
            switch (Random(19)) {
            case 0: kout << "devastated by a " << Color(3) << "tear in the fabric of space-time"; break;
            case 1: kout << "destroyed in a " << Color(11) << "Supernova Explosion"; break;
            case 2: kout << "swallowed by a " << Color(2) << "hungry sysop"; break;
            case 3: kout << "lost in a " << Color(3) << "Nebular Collapse"; break;
            case 4: kout << "lost in a " << Color(4) << "BBS Crash"; break;
            case 5: kout << "sucked into a " << Color(8) << "black hole"; break;
            case 6: kout << "destroyed in " << Color(12) << "inter-planetary warfare"; break;
            case 7: kout << "eaten by a " << Color(13) << "One-Eyed Purple Planet Eater"; break;
            case 8: kout << "towed away by a " << Color(10) << "Federation Starship"; break;
            default:
                kout << "captured by ";
                PutPirate(kout, p);
                for (int16 i = 0; i < 10; i++)
                    p->planets[i] += lost[i];
                Appeased(e, p);
                break;
            }
            kout << c_normal << "!\n";
        }
    }
}

// 6572:230c: a pirate gang, chosen by how much it hates the empire.
static int16 ChoosePirate(int16 id)
{
    int32 sum = 0;
    for (int16 k = 1; k <= 9; k++) {
        int32 h = g_pirates[k - 1].hatred[id];
        if (h > 0)
            sum += h * k;
    }
    // A random number up to sum, from two 15-bit halves (truncated to
    // 16 bits, as in the original).
    int16 hi = (int16)((sum >> 15) + 1);
    int16 rhi = (int16)(((int32)hi * b_rand()) / 0x8000);
    int16 lo = (int16)((sum & 0x7fff) + 1);
    int16 rlo = (int16)(((int32)lo * b_rand()) / 0x8000);
    int32 r = (int16)((uint16)(rhi << 15) | (uint16)rlo);
    for (int16 k = 1; k <= 9; k++) {
        int32 h = g_pirates[k - 1].hatred[id];
        if (h > 0)
            r -= h * k;
        if (r <= 0)
            return k;
    }
    return 1;
}

// A weapon for sale on the underground market.
static void Offer(Empire *e, int32 price, const char *buyprompt, const char *bought,
                  const char *cannot, int16 weapon)
{
    if (e->money.get() >= price || e->money.get() + e->bank.get() >= price) {
        if (!YesNo(2, buyprompt))
            return;
        if (e->bank.get() >= price && YesNo(1, "Withdraw funds from the bank for this purchase")) {
            e->money.add(price);
            e->bank.sub(price);
        }
        kout << bought;
        e->weapons |= weapon;
        if (e->money.get() >= price) {
            e->money.sub(price);
        } else {
            e->bank.sub(price - e->money.get());
            e->money.set(0);
        }
    } else {
        kout << cannot;
    }
}

// 6572:2459
void Advisor(Player *pl, Empire *e)
{
    if (PirateCount() > 0) {
        int16 k = ChoosePirate(e->id);
        Fate(pl, e, &g_pirates[k - 1]);
    }
    if (Random(3) == 0) {
        int16 count = ResearchCount();
        int16 idx = (int16)(((int32)count * b_rand()) / 0x8000);
        Development d;
        ResearchLoad(&d, idx);
        if (d.type < 0 || d.type > 9) {
            g_errlog << "R.index( " << idx << " ) in random() is " << d.type << "/"
                     << ResearchCount() << ".\n";
        } else if (d.cost * Planets(e, d.type)->count <= e->researchpoints &&
                   Planets(e, d.type)->count > 0 && Random(2) == 0) {
            PlanetSet *ps = Planets(e, d.type);
            ps->target = (int16)((int16)ps->target + (int16)d.Permanent());
            ps->output = (int16)((int16)ps->output + (int16)d.Temporary());
            e->researchpoints -= d.cost * Planets(e, d.type)->count;
            kout << '\n';
            d.Show();
        }
    } else if (e->researchpoints >= 10000 && !HasWeapon(e, W_MOTHERSHIP)) {
        e->weapons |= W_MOTHERSHIP;
        kout << Color(14)
             << "Your research planets develop a Mothership carrier capable of carrying 100,000 fighters into battle.\n"
             << c_normal;
    }
    if (GovCapacityGenerals(e) < 0 || GovCapacity(e) < 0) {
        int32 agents = -GovCapacity(e);
        int32 generals = -GovCapacityGenerals(e);
        if (agents < 0)
            agents = 0;
        if (e->agents < agents)
            agents = e->agents;
        if (generals < 0)
            generals = 0;
        if (e->generals < generals)
            generals = e->generals;
        if (agents > 2)
            agents /= 3;
        generals /= 3;
        e->agents -= agents;
        e->generals -= generals;
        if (agents > 0)
            kout << c_number << agents << c_normal
                 << " covert agents are killed in covert civil wars caused by spy overpopulation.\n";
        if (generals > 0)
            kout << c_number << generals << c_normal
                 << " generals kill each other in military conflicts caused by overpopulation of generals.\n";
    }
    if (g_system.regstate && r_nuclear.Bool()) {
        if ((2 * e->agents) / (TotalPlanets(e) + 200) > Random(20) && !HasWeapon(e, W_NUCLEAR) &&
            e->year > 0) {
            MaybeScreen(3, "Nuclear");
            kout << "You are approached by ";
            switch (Random(5)) {
            case 0: kout << "a sleazy trader"; break;
            case 1: kout << "Bauk'ar de Gorash,"; break;
            case 2: kout << "Torras Ysarr,"; break;
            case 3: kout << "a well dressed trader"; break;
            case 4: kout << "Diku Mudd"; break;
            }
            kout << " who offers you a " << Color(14) << "Nuclear Bomb" << c_normal << "!\n";
            Offer(e, 3000000, "Buy a nuclear bomb for 3,000,000 credits",
                  "You have bought one nuclear bomb from the underground trading market.\n",
                  "But you can't afford it right now.\n", W_NUCLEAR);
        }
    }
    if (g_system.regstate && r_chemical.Bool()) {
        if ((2 * e->agents) / (TotalPlanets(e) + 200) > Random(20) && !HasWeapon(e, W_CHEMICAL) &&
            e->year > 0) {
            MaybeScreen(3, "Chemical");
            kout << "A well-dressed trader approaches you and offers to sell a megaton of "
                 << Color(14) << "Chemicals" << c_normal << " for chemical warfare.\n";
            Offer(e, 1000000, "Buy a barrel of chemicals for 1,000,000 credits",
                  "You have bought one barrel of chemicals from the underground trading market.\n",
                  "But you can't afford it.\n", W_CHEMICAL);
        }
    }
    if (g_system.regstate && r_psionic.Bool()) {
        if ((4 * e->agents) / (TotalPlanets(e) + 200) > Random(20) && !HasWeapon(e, W_PSIONIC) &&
            e->year > 0) {
            int32 price = g_galaxy.psionicprice;
            MaybeScreen(3, "Psionic");
            kout << "A well-known smuggler approaches you and offers to sell a one-pound "
                 << Color(14) << "Psionic Bomb" << c_normal << " to you.\n";
            if (e->money.get() >= price || e->money.get() + e->bank.get() >= price) {
                kout << "Buy a psionic bomb for " << price << " credits";
                if (YesNo(2, "")) {
                    if (e->bank.get() >= price &&
                        YesNo(1, "Withdraw funds from the bank for this purchase")) {
                        e->money.add(price);
                        e->bank.sub(price);
                    }
                    kout << "You have bought a psionic bomb from the smuggler.\n";
                    e->weapons |= W_PSIONIC;
                    if (e->money.get() >= price) {
                        e->money.sub(price);
                    } else {
                        e->bank.sub(price - e->money.get());
                        e->money.set(0);
                    }
                    g_galaxy.psionicprice += 2 * g_galaxy.psionicprice + 1000000;
                }
            } else {
                kout << "You tell him that you cannot afford it.\n";
            }
        }
    }
}
