// Lotteries (segment 55e7).
#include "sre.h"

// "[MAX=n]  " after a numeric prompt.
void PutMax(int32 max)
{
    kout << c_bracket << '[' << c_bold << "MAX=" << max << c_bracket << ']' << c_normal << "  ";
}

// 55e7:0000
int32 Jackpot(int32 tickets)
{
    return 11000 * tickets + 3000000;
}

// 55e7:0713
void BuyTickets(Player *pl, Empire *e, Galaxy *g)
{
    (void)pl;
    int32 max = e->money.get() / 10000;
    if ((int32)(int16)(32000 - g->lotterytickets) < max)
        max = (int32)(int16)(32000 - g->lotterytickets);
    if ((int32)(int16)(2000 - e->tickets) < max)
        max = (int32)(int16)(2000 - e->tickets);
    if (e->year < 0 && (int32)(int16)(100 - e->tickets) < max)
        max = (int32)(int16)(100 - e->tickets);
    kout << "You currently have " << c_number << e->tickets << c_normal << " tickets.\n";
    int32 jp = Jackpot(g->lotterytickets);
    kout << "The jackpot is currently " << c_number << jp << c_normal << " credits.\n";
    kout << "At " << (int32)10000 << " credits each, how many tickets would you like to buy?  \n";
    PutMax(max);
    int16 n;
    kin >> defval(0) >> maxval(max) >> n;
    if (n <= 0)
        return;
    e->money.sub(10000 * (int32)n);
    e->tickets += n;
    g->lotterytickets += n;
    kout << "You have bought " << c_number << n << c_normal << " tickets for tomorrow's lottery.\n";
}

// 55e7:001d
void Lottery(Player *pl, Empire *e, Galaxy *g)
{
    if (g->lotterywinner == e->id) {
        kout << c_normal << "\nYou've won the " << Color(14) << "Super Daily Lottery" << c_normal << "!!!\n";
        kout << c_number << g->jackpot << c_normal << " credits has been placed in your bank account.\n\n";
    } else if (g->lotterywinner != -1 && Exists(g_empires[g->lotterywinner])) {
        Empire *w = &g_empires[g->lotterywinner];
        kout << '\n' << c_number;
        PutUntranslated(kout, String(w->name)) << c_normal;
        kout << " has won the Super Daily Lottery today!\n";
        kout << "(" << c_number << g->jackpot << c_normal << " credits)\n";
    }
    if (e->money.get() < 2000)
        return;
    if (g_system.regstate == RS_UNREGISTERED && !r_unregistered.Bool())
        return;
    kout << Color(14) << "\nPick-5 Lottery\n" << c_normal;
    kout << "Would you like to buy a lottery ticket for " << c_number << (int16)2000 << c_normal;
    if (YesNo(0, " credits")) {
        char pick[5];
        int16 matches = 0;
        int32 prize = -1;
        e->money.sub(2000);
        kout << "Enter the five digits you would like to pick:  " << c_input;
        for (int16 i = 0; i < 5;) {
            kin >> pick[i];
            if (pick[i] == '\r')
                pick[i] = (char)(Random(10) + '0');
            if (pick[i] < '0' || pick[i] > '9')
                continue;
            kout << pick[i];
            i++;
        }
        kout << c_normal << "\nThe computer picks: ";
        for (int16 i = 0; i < 5; i++) {
            kout << Color(2) << '\xfa';
            b_delay(100);
            kout << Color(10) << "\b\xf9";
            b_delay(100);
            kout << Color(11) << "\b\xfe";
            b_delay(100);
            char c = (char)(Random(10) + '0');
            kout << Color(13);
            for (int16 j = 0; j < 5; j++) {
                if (pick[j] == c) {
                    pick[j] = 0;
                    matches++;
                    kout << Color(15);
                    break;
                }
            }
            kout << '\b' << c;
        }
        kout << c_normal << '\n';
        if (matches == 0)
            prize = 0;
        if (matches == 1)
            prize = 500;
        if (matches == 2)
            prize = 10000;
        if (matches == 3)
            prize = 400000;
        if (matches == 4)
            prize = 2500000;
        if (matches == 5)
            prize = 10000000;
        if (prize > 0) {
            kout << Color(14) << "\nCongratulations!\n";
            kout << c_normal << "You have won " << c_number << prize << c_normal << " credits.\n";
            kout << "The money will be placed in your bank account.\n\n";
            if (e->bank.get() < 2000000000) {
                e->bank.add(prize);
            } else {
                kout << "The bank is full!\n";
                if (e->money.get() < 2000000000) {
                    kout << "You receive cash.\n";
                    e->money.add(prize);
                }
            }
        } else {
            kout << Color(12) << "Sorry" << c_normal;
            kout << "\nYou didn't win this time.  Better luck next time!\n\n";
        }
    }
    if (g_system.regstate != RS_UNREGISTERED && YesNo(1, "Would you like to buy Super Daily Lottery Tickets"))
        BuyTickets(pl, e, g);
}
