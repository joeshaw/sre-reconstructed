// The bank (segment 3e41) and investments in bonds (segment 1719).
#include "sre.h"
#include "events.h"

static Resource r_investments("Bank.Investments", (int32)1);              // 0988
static Resource r_playerinvest("Bank.PlayerInvestments", (int32)1);       // 46fe

static void Key(char c)
{
    kout << c_bracket << '[' << c_bold << c << c_bracket << ']' << c_normal;
}

// "[MAX=n] " after a prompt.
static void Max1(int32 max)
{
    kout << c_bracket << '[' << c_bold << "MAX=" << max << c_bracket << ']' << c_normal << ' ';
}

// 1719:000a
static void TreasuryBonds(Empire *e, Galaxy *g)
{
    int32 max = e->money.get() / 8500;
    int16 turns = 5;
    Money m;
    m.v = 0;
    m.set(8500000);
    int16 i;
    for (i = 0; i < 200 && m.get() < 10000000; i++)
        m.add((int32)g->interest * (m.get() / (100 * (int32)g->grantturns)));
    i /= 2;
    if (i > turns)
        turns = i;
    if (max > 3392)
        max = 3392;
    kout << Color(12) << " * " << c_input << "Galactic Treasury Bonds" << c_normal << '\n';
    HLine(Color(4), 0x28);
    kout << "For 8,500 credits, you can buy Galactic Treasury Bonds, ";
    kout << "which return 10,000 credits after " << turns;
    kout << " turns, or 1 (real) hour, whichever is later.\n";
    kout << "How many would you like?  ";
    Max1(max);
    int32 n;
    kin >> defval(0) >> maxval(max) >> n;
    if (n <= 0)
        return;
    EventBank ev(e->id, "Galactic Treasury Bonds", 10000 * n, (int16)(e->year + turns), -1);
    ev.SetDue(0, 1, 0);
    AddEvent(&ev);
    e->money.sub(8500 * n);
    kout << c_normal << "You will be notified when your " << c_number << n << c_normal
         << " bonds mature.  (Sometime after " << "year " << (int16)(e->year + turns) << ".)\n";
}

// 1719:1067
struct InvestPrompt : MenuHook {
    Empire *e;
    explicit InvestPrompt(Empire *x) : e(x) {}
    int16 Call()
    {
        kout << "You have " << c_number << e->money.get() << c_normal << " credits.\n";
        return 1;
    }
};

// 1719:0394: buy the Treasury's or another empire's bonds.
static void MakeInvestment(Empire *e, Galaxy *g)
{
    Menu m("Investments.Buy", "Make an Investment", 0xf, 3, 2, 0xe, 7, 7, 7, 7, 7, 7);
    m.prompt = new InvestPrompt(e);
    m.Add("Galactic Treasury Bonds", '$', 0x32);
    for (int16 i = 0; i < 25; i++) {
        Empire *b = &g_empires[i];
        if (b->bondprice <= 0)
            continue;
        char key = 0;
        if (b->bonds < b->maxbonds && e->id != i)
            key = (char)('A' + i);
        m.Add(b->bondname, key, i);
    }
    if (m.count > 7)
        m.columns = 2;
    if (m.count > 14)
        m.columns = 3;
    int16 r = r_playerinvest.Bool() ? m.Run(1, -1) : 0x32;
    if (r == -1)
        return;
    if (r == 0x32) {
        TreasuryBonds(e, g);
        return;
    }
    Empire *b = &g_empires[r];
    int32 max = e->money.get() / b->bondprice;
    if ((int32)(int16)(b->maxbonds - b->bonds) < max)
        max = (int16)(b->maxbonds - b->bonds);
    kout << Color(9) << " * " << c_input << b->bondname << c_normal << '\n';
    HLine(Color(1), 0x28);
    kout << c_normal;
    kout << "These bonds are available for " << c_number << b->bondprice << c_normal
         << " credits.  They pay " << c_number << b->bondpayoff << c_normal
         << " credits after either " << b->bondturns << " turns or " << b->bondhours
         << " (real) hours, whichever is later.\n";
    kout << "How many would you like?  ";
    Max1(max);
    int32 n;
    kin >> defval(0) >> maxval(max) >> n;
    if (n <= 0)
        return;
    EventBank ev(e->id, b->bondname, n * b->bondpayoff, (int16)(e->year + b->bondturns), r);
    ev.SetDue(0, b->bondhours, 0);
    AddEvent(&ev);
    int32 cost = n * b->bondprice;
    int32 oldmoney = b->money.get();
    b->money.add(cost);
    if (b->money.get() - cost != oldmoney) {
        // The issuer's treasury is full: the rest goes to its bank account.
        int32 rest = cost - (b->money.get() - oldmoney);
        int32 oldbank = b->bank.get();
        b->bank.add(rest);
        if (b->bank.get() - rest != oldbank) {
            // (The original measures the overflow against the buyer's bank.)
            int32 over = rest - (e->bank.get() - oldbank);
            int32 back = (over + b->bondprice - 1) / b->bondprice;
            n -= back;
            cost -= back * b->bondprice;
            b->bank.sub(back * b->bondprice - over);
            kout << "Your financial team could only acquire " << n << " bonds at this time.\n";
        }
    }
    e->money.sub(cost);
    b->bonds = (int16)(b->bonds + n);
    kout << c_normal << "You will be notified when your " << c_number << n << c_normal
         << " bonds mature.  (Sometime after " << "year " << (int16)(e->year + b->bondturns)
         << ".)\n";
}

// 1719:0b7d: set up the empire's own bonds.
static void ConfigureBonds(Empire *e)
{
    if (e->bondprice == 0) {
        kout << "You are not currently offering any bonds.\n";
        if (!YesNo(0, "Do you wish to offer bonds"))
            return;
        if (e->bondname[0] == 0) {
            strcpy(e->bondname, "- Bonds");
            e->bondname[0] = (char)(e->id + 'A');
            e->bondprice = 8500;
            e->bondpayoff = 10000;
            e->bondturns = 10;
            e->bondhours = 1;
            e->maxbonds = 1;
        }
    }
    kout << "Your empire currently offers " << c_input << e->bondname << c_normal << " for "
         << c_number << e->bondprice << c_normal << " credits, which pays " << c_number
         << e->bondpayoff << c_normal << " credits when it matures.\n";
    kout << "The bonds mature after after " << e->bondturns << " turns and " << e->bondhours
         << " (real) hours.\n";
    if (YesNo(0, "Do you wish to change the name")) {
        char buf[0x20];
        strcpy(buf, e->bondname);
        kout << "Enter a name for your bonds: (please make it plural)\n> ";
        kin >> maxlen(0x1f);
        kin.GetLine(buf);       // edits the current name
        if (buf[0])
            strcpy(e->bondname, buf);
    }
    kout << "How much should each bond cost?  ";
    kin >> maxval(1000000) >> defval(e->bondprice) >> e->bondprice;
    kout << "How much should be paid when the bond matures?  ";
    kin >> maxval(2000000) >> defval(e->bondpayoff) >> e->bondpayoff;
    kout << "You can set the maturity date in terms of game turns, (real) hours, or both.  Set 0 if you do not want one or the other.\n";
    for (;;) {
        kout << "What is the number of turns before maturity?  ";
        kin >> maxval(100) >> defval((int32)e->bondturns) >> e->bondturns;
        kout << "What is the minimum number of hours before maturity?  ";
        kin >> maxval(240) >> defval((int32)e->bondhours) >> e->bondhours;
        if (e->bondturns != 0 || e->bondhours != 0)
            break;
        kout << "You must set " << Color(12) << "either" << c_normal
             << " the number of turns or the numbers of hours.\n";
    }
    if (30 * ((e->bondpayoff - e->bondprice) / (int32)(int16)(e->bondturns * 3 + e->bondhours)) >
        e->bondprice) {
        kout << "The Galactic Coordinator does not permit you to create such a bond.  It thinks your bonds are too profitable.  Try lowering your payoff and try again.\n";
        e->bondprice = 0;
        return;
    }
    kout << "How many bonds can be bought (maximum) at a time?  ";
    kin >> maxval(1000) >> defval((int32)e->maxbonds) >> e->maxbonds;
    if (e->bondhours == 0 && e->bondturns == 0) {
        kout << "You have specified 0 for both turns and hours.\n";
        e->bondprice = 0;
    }
}

// 1719:10b8
static void Investments(Player *p, Empire *e, Galaxy *g)
{
    (void)p;
    if (!r_playerinvest.Bool()) {
        MakeInvestment(e, g);
        return;
    }
    for (;;) {
        Menu m("Investments", "Investments", 0xf, 3, 2, 0xe, 0xa, 7, 7, 7, 7, 7);
        m.prompt = new InvestPrompt(e);
        m.Add("List investments", 'S', 1);
        m.Add("Make an investment", 'I', 2);
        m.Add("Configure your bonds", 'C', 3);
        if (e->bondprice > 0)
            m.Add("Kancel bond offers", 'K', 4);
        int16 r = m.Run(1, -1);
        if (r == -1)
            return;
        if (r == 1)
            ListInvestments(e->id);
        if (r == 2)
            MakeInvestment(e, g);
        if (r == 3)
            ConfigureBonds(e);
        if (r == 4 && YesNo(0, "Do you wish to stop offering bonds"))
            e->bondprice = 0;
    }
}

// 3e41:0000
void Bank(Player *p, Empire *e, Galaxy *g)
{
    if (!g_system.regstate && g_door.port) {
        // Shown on the sysop's screen only, with a beep.
        klocal << Color(14) << "You are not allowed to visit the bank in the unregistered versions!\n"
               << c_normal;
        return;
    }
    kout << c_normal << "\nWelcome to " << Color(14) << Untr(g_system.bank);
    char c;
    do {
        kout << '\n';
        int32 credit = 35 * (int32)e->year + 3 * e->soldiers + 8 * e->fighters +
                       10 * e->stations + 10 * e->heavycruisers + 13 * e->carriers +
                       e->food / 31 + 230 * e->planets[P_TOURISM].count +
                       500 * e->planets[P_ORE].count + 500 * e->planets[P_FOOD].count +
                       400 * e->planets[P_SUPPLY].count + 230 * e->planets[P_EDUCATION].count +
                       330 * e->planets[P_GOVERNMENT].count + 50 * e->planets[P_RESEARCH].count +
                       230 * e->planets[P_URBAN].count + 1100 * e->planets[P_ANTIPOLLUTION].count +
                       150000;
        kout << c_normal << "You have " << c_number << e->money.get() << c_normal << " credits, and ";
        if (e->bank.get() >= 0)
            kout << c_number << e->bank.get() << c_normal << " credits in the bank.\n";
        else
            kout << "you have a loan of " << c_number << -e->bank.get() << c_normal << " credits.\n";
        if (e->bank.get() > 0) {
            Key('W');
            kout << "ithdraw, ";
        } else {
            kout << "Take a ";
            Key('L');
            kout << "oan, ";
        }
        if (r_investments.Bool()) {
            Key('I');
            kout << "nvest, ";
            Key('B');
            kout << "onds, ";
        }
        Key('D');
        kout << "eposit, or ";
        Key('Q');
        kout << "uit?  " << c_input;
        do {
            c = GetChoice("QDWLBIS", 1);
            if (c == 'W' && e->bank.get() <= 0)
                c = 0;
            if (c == 'L' && e->bank.get() > 0)
                c = 0;
            if (!r_investments.Bool() && (c == 'I' || c == 'B'))
                c = 0;
        } while (c == 0);
        if (!g_system.regstate)
            c = 0;
        if (c == 'S') {
            kout << "See Investments" << c_normal << '\n';
            ListInvestments(e->id);
        }
        if (c == 'I') {
            kout << "Invest" << c_normal << '\n';
            TreasuryBonds(e, g);
        }
        if (c == 'D') {
            kout << "Deposit" << c_normal << '\n';
            int32 max = e->money.get();
            if (e->bank.get() >= 0 && 2000000000 - e->bank.get() < max)
                max = 2000000000 - e->bank.get();
            if (e->bank.get() >= 0) {
                kout << "At a " << c_number << g->interest << c_normal;
                kout << "% daily interest rate,\nhow much do you wish to deposit?  ";
                Max1(max);
            } else {
                kout << "You have loaned " << c_number << -e->bank.get() << c_normal << " credits at a ";
                kout << c_number << g->interest << c_normal << "% annual interest rate.\n";
                kout << "How much do you wish to pay back?  ";
            }
            int32 n;
            kin >> defval(0) >> maxval(max) >> n;
            e->money.sub(n);
            e->bank.add(n);
        }
        if (c == 'B') {
            kout << "Bonds" << c_normal << '\n';
            Investments(p, e, g);
        }
        if (c == 'W' && e->bank.get() > 0) {
            kout << "Withdraw" << c_normal << '\n';
            kout << "How much do you wish to withdraw?  ";
            Max1(e->bank.get());
            int32 max = e->bank.get();
            if (2000000000 - e->money.get() < e->bank.get())
                max = 2000000000 - e->money.get();
            int32 n;
            kin >> defval(0) >> maxval(max) >> n;
            if (e->money.get() > 0 && MoneyOverflow(e->money.get(), n)) {
                kout << "[SRE can only handle numbers up to " << (int32)2000000000
                     << ", so some money will have to stay in the bank.]\n";
                while (e->money.get() < 1990000000 && n > 10000000) {
                    e->money.add(10000000);
                    n -= 10000000;
                    e->bank.sub(10000000);
                }
                while (e->money.get() < 2000000000 && n > 10000) {
                    e->money.add(10000);
                    n -= 10000;
                    e->bank.sub(10000);
                }
            } else {
                e->bank.sub(n);
                e->money.add(n);
            }
        }
        if (c == 'L' && e->bank.get() <= 0) {
            int32 payment = 0;
            kout << "Loan" << c_normal << '\n';
            if (credit + e->bank.get() <= 0) {
                kout << "Your credit is overdrawn.\n";
            } else {
                Money avail;
                avail.v = 0;
                avail.set(credit);
                avail.add(e->bank.get());
                if (TotalPlanets(e) + 8 * e->planets[P_PETROLEUM].count <
                    100 * e->planets[P_ANTIPOLLUTION].count) {
                    kout << "This bank rewards environmentalists who have set up anti-pollution planets; you have a better credit rating with us.\n";
                    // (The original raises a copy of the limit that is never used.)
                }
                kout << "At a " << c_number << g->interest << c_normal;
                kout << "% annual interest rate,\nhow much do you wish to borrow?  ";
                Max1(avail.get());
                int32 n;
                kin >> defval(0) >> maxval(avail.get()) >> n;
                if (n > 0) {
                    kout << "That will require a " << c_number << (int32)g->interest * (n / 100)
                         << c_normal << " finance charge.\n";
                    payment = (int32)g->interest * (n / 70) + 1;
                    kout << "The yearly payment is " << c_number << payment << c_normal
                         << " credits.\n";
                    if (!YesNo(0, "Are you sure you want a loan")) {
                        payment = 0;
                        n = 0;
                    }
                }
                if (MoneyOverflow(n, e->money.get()) && e->money.get() > 0) {
                    kout << "The bank is unable to find the full sum of money for loan.\n";
                    while (e->money.get() < 1990000000 && n > 10000000) {
                        e->money.add(10000000);
                        n -= 10000000;
                        e->bank.sub(100000 * (int32)g->interest + 10000000);
                    }
                    while (e->money.get() < 2000000000 && n > 10000) {
                        e->money.add(10000);
                        n -= 10000;
                        e->bank.sub(100 * (int32)g->interest + 10000);
                    }
                } else {
                    e->bank.sub(n + n * (int32)g->interest / 100);
                    e->money.add(n);
                    e->d1a4 += payment;
                }
            }
        }
    } while (c != 'Q');
    kout << "Quit" << c_normal << "\n\n";
}
