// Hourly and daily maintenance, and the newspaper (segment 5681).
#include "sre.h"
#include "events.h"
#include <fcntl.h>
#include <unistd.h>


static Resource r_deletedead("DeleteDeadPlayers", (int32)1);       // 5544
static Resource r_ptsolar("Points.SolarMaster", (int32)5);         // 5550
static Resource r_ptweapons("Points.WeaponsMaster", (int32)1);     // 555c
static Resource r_ptplanet("Points.PlanetMaster", (int32)1);       // 5568

// 5681:0000
static void Status(const char *msg)
{
    PutUntranslated(kout, String(msg));
}

// 5681:004a: erase a status message.
static void StatusEnd(const char *msg)
{
    for (int16 i = 0; msg[i]; i++)
        kout << "\b \b";
}

// 5681:0079
static void Backup(const char *from, const char *to)
{
    int in = open(HostPath(from).c_str(), O_RDONLY);
    if (in == -1)
        return;
    int out = open(HostPath(to).c_str(), O_WRONLY | O_CREAT | O_TRUNC, 0666);
    if (out == -1) {
        g_errlog << "Could not generate backup file for " << from << ".\n";
        close(in);
        return;
    }
    char buf[0x400];
    for (;;) {
        ssize_t n = read(in, buf, sizeof buf);
        if (n <= 0)
            break;
        if (write(out, buf, (size_t)n) != n)
            g_errlog << "Error while backing up " << from << ".\n";
    }
    close(in);
    close(out);
}

// 5681:0160: one of three newspapers, with random headlines.
static void Newspaper()
{
    int16 paper = g_galaxy.newspaper;
    if (paper == 2) {
        g_log2 << '\\' << Color(14) << "   Farmer Bob's Guide to the Galaxy        " << Color(7)
               << "Printed Annually\n";
        g_log2 << '\\' << Color(12)
               << "\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4"
                  "\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4"
                  "\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4"
                  "\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\n\n"
               << c_normal;
        if (Random(3) == 0) {
            switch (Random(3)) {
            case 0: g_log2 << "It looks like a good year for crops.\n"; break;
            case 1:
                g_log2 << "The space storm passing through this part of the galaxy may lead to crop failures!\n";
                break;
            case 2: g_log2 << "Eat your broccoli!\n"; break;
            }
        }
        if (Random(7) == 0) {
            switch (Random(4)) {
            case 0: g_log2 << "This year will be great for Arcadian sheep farmers.\n"; break;
            case 1:
                g_log2 << "Galactic tollway " << (int16)(Random(999) + 1)
                       << "'s construction may disrupt Unkulian onyx grazing planets.\n";
                break;
            case 2:
                g_log2 << "Overpopulation of Regalian Redfoxes may lead to lower fur prices.\n";
                break;
            case 3: g_log2 << "No news is good gnus.\n"; break;
            }
        }
        if (Random(10) == 0) {
            g_galaxy.newspaper = 3;
            g_log2 << "Centurion Rahad has come out of retirement to take over this news agency.\n";
        }
    } else if (paper == 3) {
        g_log2 << '\\' << Color(14) << "   Centurion Rahad's Galactic Times        " << Color(7);
        PutTime(g_log2, g_today) << '\n';
        g_log2 << '\\' << Color(10)
               << "\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd"
                  "\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd"
                  "\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd"
                  "\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\xcd\n\n"
               << c_normal;
        if (Random(5) == 0) {
            switch (Random(3)) {
            case 0:
                g_log2 << "The Yip have been improving their weapon systems, much to the dismay of the Galactic Coordinator.\n";
                break;
            case 1: {
                char letter = (char)(Random(26) + 'A');
                int16 num = (int16)(Random(9999) + 1);
                g_log2 << "Science Station " << num << letter
                       << " is being converted into a military outpost.\n";
                break;
            }
            case 2:
                g_log2 << "Military hardware prices may go up as skirmishes in the inner Antarean sector heat up.\n";
                break;
            }
        }
        if (Random(5) == 0) {
            switch (Random(4)) {
            case 0:
                if (g_galaxy.planetmaster != -1)
                    g_log2 << c_empire << Untr(g_empires[g_galaxy.planetmaster].name) << c_normal
                           << " appears to be a good military target.\n";
                break;
            case 1:
                if (g_galaxy.weaponsmaster != -1)
                    g_log2 << c_empire << Untr(g_empires[g_galaxy.weaponsmaster].name)
                           << c_normal << " would be a strong opponent.\n";
                break;
            case 2:
                if (g_galaxy.solarmaster != -1)
                    g_log2 << "Computer psychohistory models predict that " << c_empire
                           << Untr(g_empires[g_galaxy.solarmaster].name) << c_normal
                           << " can be defeated by a united offense.\n";
                break;
            case 3: g_log2 << "All is quiet, unfortunately.\n"; break;
            }
        }
        if (Random(10) == 0) {
            g_galaxy.newspaper = 1;
            g_log2 << "Centurion Rahad is considering retiring!\n";
        }
    } else {
        g_log2 << '\\' << Color(11)
               << "   The Galactic Star                                         " << Color(7);
        PutTime(g_log2, g_today) << '\n';
        g_log2 << '\\' << Color(9)
               << "___________________________________________________________________________\n\n"
               << c_normal;
        if (Random(10) == 0) {
            if (Random(3) == 0) {
                g_log2 << "The Galactic Coordinator steals " << Color(12) << "Reko's" << Color(7)
                       << " car!\n";
            } else if (Random(3) == 0) {
                g_log2 << "Covert agents kidnap " << Color(12) << "Reko's" << Color(7)
                       << " daughter!\n";
            } else if (Random(3) == 0) {
                g_log2 << "Rumors spread that " << Color(12) << "Emperor Reko" << Color(7)
                       << " has picked Abraham Lincoln\n     as heir to his throne!\n";
            } else if (Random(10) == 0) {
                g_log2 << Color(14) << "Ross Perot" << Color(7)
                       << " denounces the use of free trade agreements.\n";
            }
        }
        if (Random(10) == 0) {
            if (Random(3) == 0)
                g_log2 << "A band of pirates terrorizes the galaxy, news at 11.\n";
            else if (Random(3) == 0)
                g_log2 << "Bank managers complain as the number of loans has taken a sharp turn downwards.\n";
            else if (Random(3) == 0)
                g_log2 << "Lottery officials were found 'fixing' the numbers!\n";
            else if (Random(10) == 0)
                g_log2 << "Rumors are that investments are a good buy.\n";
        }
        if (Random(10) == 0) {
            g_galaxy.newspaper = 2;
            g_log2 << "The Galactic Star is on the verge of bankrupcy.  Can another newspaper take over?\n";
        }
    }
}

// 1c59:0855: planets come back onto the market over time.
void RegeneratePlanets(Galaxy *g)
{
    int32 now = b_time(0);
    if (g->marketplanets < 0)
        g->marketplanets = 0;
    if (now > g->lastregen && g->regenseconds > 0) {
        if (g->marketplanets < 30000) {
            int32 n = (now - g->lastregen) / ((int32)g->regenseconds + 1);
            if (n > 0) {
                g->marketplanets += n;
                g->lastregen = g->lastregen + n * (int32)g->regenseconds;
            }
        } else {
            g->lastregen = now;
        }
    }
}

// 5681:1f8f: a new day for an empire.
static void DailyEmpire(Empire *e)
{
    GalaxyOpen();
    e->turnsperday = g_galaxy.grantturns;
    e->turnsused = 0;
    e->inactivedays--;
    e->attacks = 0;
    e->bombmask = 0;
    e->hostagemask = 0;
    e->setupmask = 0;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (e->treaty[i].days > 0)
            e->treaty[i].days = (uint8)(e->treaty[i].days - 1);
    }
    if (r_deletedead.Bool() &&
        !(Exists(*e) && TotalPlanets(e) > 0 && e->population > 0)) {
        g_log2 << "Player " << c_empire << Untr(e->name) << c_normal << " has ";
        if (TotalPlanets(e) == 0 && e->population == 0)
            g_log2 << "abdicated!\n";
        else
            g_log2 << "collapsed!\n";
        DeleteEmpire(e);
        EmpireInit(e);
    } else if (r_deletedead.Bool() && e->inactivedays <= 0) {
        int32 days = r_inactivetimeout.Long();
        g_log2 << "Empire " << c_empire << Untr(e->name) << c_normal << " erased - inactive for "
               << days << " days.\n";
        DeleteEmpire(e);
        EmpireInit(e);
    }
    GalaxyClose();
}

// 5681:0c52
void Maintenance(Galaxy *g)
{
    GalaxyOpen();
    if (g_today > g->enddate)
        return;
    RegeneratePlanets(g);
    const char *msg = 0;
    if (g_now - g->lasthourly >= 3600) {
        msg = "Running hourly maintenance...";
        Status(msg);
        if (Midnight(g->lasthourly) < g->created)
            g->lasthourly = g->created;
        while (g_now - g->lasthourly > 3600) {
            for (int16 i = 1; i <= 9; i++)
                PirateHourly(&g_pirates[i - 1]);
            g->lasthourly = g->lasthourly + 3600;
        }
        StatusEnd(msg);
    }
    if (g_today - Midnight(g->created) < 86400 * (int32)g->grantdays) {
        GalaxyClose();
        return;
    }
    PutUntranslated(kout, String("Running daily maintenance...\n"));
    msg = "Making backups...";
    Status(msg);
    HostMkdir("Backup");
    Backup("Data/System.II", "Backup/System.II");
    Backup("Data/Empire.II", "Backup/Empire.II");
    Backup("Data/Galaxy.II", "Backup/Galaxy.II");
    Backup("Data/Player.II", "Backup/Player.II");
    Backup("Data/Pirate.II", "Backup/Pirate.II");
    Backup("Data/Battles.II", "Backup/Battles.II");
    Backup("Data/Trading.II", "Backup/Trading.II");
    Backup("Data/CovertOp.II", "Backup/CovertOp.II");
    Backup("Data/Message.II", "Backup/Message.II");
    Backup("Data/Report.II", "Backup/Report.II");
    Backup("Data/Event.II", "Backup/Event.II");
    StatusEnd(msg);

    msg = "Updating news files...";
    Status(msg);
    HostMkdir("News");
    HostRemove("News/NEWS9.MSG");
    for (int16 i = 8; i >= 0; i--) {
        char from[] = "NEWS/NEWS_.MSG";
        char to[] = "NEWS/NEWS_.MSG";
        from[9] = (char)(i + '0');
        to[9] = (char)(i + '1');
        HostRename(from, to);
    }
    for (int16 i = 0; i <= 9; i++)
        News(i);
    Newspaper();
    StatusEnd(msg);
    ProcessEvents();

    msg = "Searching for illegal nuclear bomb traders...";
    Status(msg);
    if (g->nukelimit > 400)
        g->nukelimit += 2;
    else if (g->nukelimit > 300)
        g->nukelimit += 5;
    else
        g->nukelimit += 15;
    if (g->nukepercent < 30)
        g->nukepercent++;
    g->psionicprice = 15 * (g->psionicprice / 20);
    if (g->psionicprice < 5000000)
        g->psionicprice = 5000000;
    StatusEnd(msg);

    msg = "Checking top scores and lottery...";
    Status(msg);
    int32 bestscore = 10000, bestplanets = 300, bestworth = 50000;
    int16 oldsolar = g->solarmaster;
    int16 oldplanet = g->planetmaster;
    int16 oldweapons = g->weaponsmaster;
    g->solarmaster = -1;
    g->weaponsmaster = -1;
    g->planetmaster = -1;
    int16 tickets = g->lotterytickets;
    int16 winner = (int16)(((int32)tickets * b_rand()) / 0x8000);
    if (g->lotterytickets <= 0) {
        winner = -1;
        g->lotterytickets = 0;
    }
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *e = &g_empires[i];
        if (!Exists(*e))
            continue;
        if (e->lastplay < g->startdate)
            e->lastplay = g->startdate;
        DailyEmpire(e);
    }
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *e = &g_empires[i];
        if (!Exists(*e))
            continue;
        if (winner >= 0 && e->tickets > winner) {
            g->lotterywinner = e->id;
            g->jackpot = Jackpot((int32)g->lotterytickets);
            if (e->bank.get() < 2000000000)
                e->bank.add(g->jackpot);
            else if (e->money.get() < 2000000000)
                e->money.add(g->jackpot);
            g->lotterytickets = 0;
            winner = -1;
        }
        if (winner >= 0)
            winner = (int16)(winner - e->tickets);
        e->tickets = 0;
        if (EmpireScore(e) > 3 * bestscore / 2) {
            bestscore = EmpireScore(e);
            g->solarmaster = e->id;
        } else if (EmpireScore(e) >= bestscore) {
            g->solarmaster = -1;
            bestscore = EmpireScore(e);
        }
        if (TotalPlanets(e) > 3 * bestplanets / 2) {
            bestplanets = TotalPlanets(e);
            g->planetmaster = e->id;
        } else if (TotalPlanets(e) >= bestplanets) {
            g->planetmaster = -1;
            bestplanets = TotalPlanets(e);
        }
        if (NetWorth(e) > 3 * bestworth / 2) {
            bestworth = NetWorth(e);
            g->weaponsmaster = e->id;
        } else if (NetWorth(e) >= bestworth) {
            g->weaponsmaster = -1;
            bestworth = NetWorth(e);
        }
    }
    g->lotterytickets = 0;
    if (g->solarmaster != -1) {
        if (g->coordfund < 0)
            g->coordfund = 0;
        Empire *e = &g_empires[g->solarmaster];
        e->points += r_ptsolar.Long();
        g_log2 << c_empire << Untr(e->name) << c_normal << " has been ";
        if (g->solarmaster == oldsolar)
            g_log2 << "once again ";
        g_log2 << "chosen today's Solar Master, and has received a ";
        g_log2 << c_number << 10 * g->coordfund << c_normal << " credit bonus!\n";
        e->money.add(10 * g->coordfund);
        g->coordfund -= g->coordfund / 100;
    }
    if (g->planetmaster != -1) {
        Empire *e = &g_empires[g->planetmaster];
        e->points += r_ptplanet.Long();
        g_log2 << c_empire << Untr(e->name) << c_normal;
        if (g->planetmaster == oldplanet)
            g_log2 << " is still the";
        else
            g_log2 << " has been chosen today's";
        g_log2 << " Planet Master!\n";
    }
    if (g->weaponsmaster != -1) {
        int16 lost = 0;
        Empire *e = &g_empires[g->weaponsmaster];
        e->points += r_ptweapons.Long();
        if (g->weaponsmaster != oldweapons)
            g_log2 << c_empire << Untr(e->name) << c_normal
                   << " has been chosen the new Weapons Master";
        if (TotalPlanets(e) > 500) {
            for (int16 i = 0; i < MAXEMPIRES; i++) {
                if (e->treaty[i].soldierpct() > 0 || e->treaty[i].cruiserpct() > 0) {
                    Empire *o = &g_empires[i];
                    uint8 days = e->treaty[i].days;
                    lost = 1;
                    SetTreaty(o->treaty[e->id], 1, days);
                    SetTreaty(e->treaty[o->id], 1, days);
                }
            }
        }
        if (g->weaponsmaster != oldweapons) {
            if (lost)
                g_log2 << ", and lost some defense pacts";
            g_log2 << "!\n";
        }
    }
    StatusEnd(msg);
    if (g_system.regstate != 0) {
        if (g->interest < 3) {
            g->interest += 2;
            g_log2 << "Interest rates go up to " << g->interest
                   << "% as the galaxy's economy booms!\n";
        } else if (g->interest > 10) {
            g->interest -= 3;
            g_log2 << "Interest rates fall sharply to " << g->interest
                   << "% as pirate raids increase!\n";
        } else if (Random(8) == 0) {
            g->interest -= 2;
            g_log2 << "The galaxy's interest rates fall to " << g->interest
                   << "% in response to a weakening economy.\n";
        } else if (Random(5) == 0) {
            g->interest--;
            g_log2 << "Bank interest rates fall slightly to " << g->interest << "%.\n";
        } else if (Random(3) == 0) {
            g->interest++;
            g_log2 << "Bank interest rates rise to " << g->interest << "%.\n";
        }
    }
    g->created = g_today;
    g->lasthourly = g_today;
    GalaxyClose();
}
