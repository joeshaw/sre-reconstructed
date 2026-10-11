// Solar Realms Elite: declarations shared by the game modules.
#ifndef SRE_SRE_H
#define SRE_SRE_H

#include "types.h"
#include "brtl.h"
#include "stream.h"
#include "sstring.h"
#include "files.h"
#include "resource.h"
#include "devices.h"
#include "menu.h"
#include "screens.h"
#include "translate.h"
#include "timeutil.h"
#include "game.h"
#include "crypt.h"
#include <string.h>
#include <stdlib.h>

// Borland's random(n), as the game computes it inline.
inline int16 Random(int16 n) { return (int16)(((int32)b_rand() * n) / 0x8000); }

// Logs.
extern OStream &g_log2;          // DS:d138 news stream
extern OStream &g_errlog;        // DS:d160 SYSOP.LOG
extern Color c_white;           // 6782
extern Color c_6792, c_6796, c_679a, c_679e, c_67a2;
extern Color c_green;           // 0094

// main (1848)
extern Resource r_unregistered;     // 50d0
extern Resource r_flavor;           // 50dc

// system (6c38)
void SystemLoad();
void SystemClearInUse();
void SystemInit();
void SystemSetSysop(const String &s);
void SystemSetBBS(const String &s);
void SystemShowInfo();
void SystemMaintenance();
int16 SystemValid();
int16 SystemLoadData();
void SystemSave();
void SystemSaveAll();
void MainMenu();
void Rankings();

// screens globals
extern int16 g_sbcolor;             // a6e6
extern int16 g_statusbars;          // a6e8

// galaxy (5222)
void GalaxyOilPrice(Galaxy *g);
void GalaxySetName(Galaxy *g, const String &s);
void ScoreListAll(Galaxy *g);
int16 MaskTest(int32 mask, int16 bit);
void MaskSet(int32 *mask, int16 bit);
void MaskClear(int32 *mask, int16 bit);
// Covert operations (the menu's items are one more).
enum {
    OP_SPY, OP_INSURGENTAID, OP_SETUP, OP_DISSENSION, OP_DEMORALIZE, OP_BOMBING,
    OP_INFILTRATION, OP_HOSTAGES, OP_BRIBE
};
int32 *OpMask(Empire *e, int16 op);     // 14f6:05dc
void GalaxyInit(Galaxy *g);
void GalaxyReset(Galaxy *g);
void GalaxyOpen();
void GalaxyClose();
void ScoreList(Galaxy *g, int32 mask, OStream *o);
int32 EmpireMask();
int32 ProtectedMask();
int16 GalaxyPlayable(Galaxy *g);
void ShowMasters(Galaxy *g);
int16 SearchEmpire(String search, int32 allowed);
int16 ChooseEmpires(int32 allowed, int32 *chosen, String exits, void (*list)(int32),
                    const char *prompt);
int16 ChooseEmpire(int32 allowed);
extern Resource r_inactivetimeout, r_failedcovert, r_attackvictim, r_pirateraids, r_civilwar;

// empire (48cc, 1c59)
void EmpireInit(Empire *e);
int32 TotalPlanets(Empire *e);
int32 NetWorth(Empire *e);
int32 EmpireScore(Empire *e);
PlanetSet *Planets(Empire *e, int16 type);
void EmpireInitId(Empire *e, int16 id);
void EmpireSetName(Empire *e, const String &s);
int32 OilPrice();
int16 EmpireOfPlayer(int16 player);
void WrapAt(int16 n);
void StatusItem(int32 v, const char *name, int16 flag);
void EmpireStatus(Empire *e, int16 full);
void ShowStatus(Empire *e);
int32 ProtectionBonus(Empire *e);
void NextPhase(Empire *e);
void PrevPhase(Empire *e);
void BuildCmdShip(Empire *e);
void BankInterest(Empire *e);
void PlanetsMoveToward(Empire *e);
void UpdateScore(Empire *e);
void DriftFavor(Empire *e);
int32 GovCapacity(Empire *e);
int32 GovCapacityGenerals(Empire *e);
const char *InsurgencyName(Empire *e);
const char *PlanetName(int8 type);
void InsurgencyUp(Empire *e);
int32 FoodNeeded(Empire *e, int16 who);  // 512f:0105
void InsurgencyDown(Empire *e);
void ClearShortages(Empire *e);
void SetTreaty(Treaty &t, int16 type, uint8 days);

// spending (1af6)
int32 PriceSoldiers(Empire *e);
int32 PriceFighters(Empire *e);
int32 PriceStations(Empire *e);
int32 PriceGenerals(Empire *e);
int32 PriceCmdShip(Empire *e);
int32 PricePlanet(Empire *e);
int32 PriceAgents(Empire *e);
int32 PriceHeavyCruisers(Empire *e);
int32 PriceCarriers(Empire *e);

// pirates (4b60)
void PirateInit(Pirate *p);
void PirateNew(Pirate *p);
int32 PiratePlanets(Pirate *p);
int32 PirateNetWorth(Pirate *p);
void PirateHourly(Pirate *p);
void CreatePirates();
int16 PirateCount();
const char *PirateName(Pirate *p);
uint8 PirateColor(Pirate *p);
OStream &PutPirate(OStream &o, Pirate *p);

// player (64d2)
void PlayerReset(Player *p);
void PlayerLoad(Player *p, int16 index);
void PlayerSave(Player *p);
int16 PlayerFind(const String &user);
void PlayerWhoAmI(Player *p);
String TreatyName(int16 type);                 // 14f6:044e
String Pronoun(int16 type, int16 male);
String PlayerPronoun(Player *p, int16 type);
String RandomPronoun(int16 type);
int16 PlayerFree();
void PlayerList();
void PlayerSetName(Player *p, const String &s);
void PlayerSetUser(Player *p, const String &s);

// news (5b6a)
int16 ShowNews(OStream &o, IStream *in, const String &file, int32 since);
int16 ShowNews(const String &file, int32 since);
char GetChoice(const char *valid, int16 upper);

// registration (68a3)
int16 Register();

// lottery (55e7)
void PutMax(int32 max);
int32 Jackpot(int32 tickets);
void Lottery(Player *pl, Empire *e, Galaxy *g);

// misc modules
void Maintenance(Galaxy *g);                // 5681:0c52
void GalaxyMenu(Galaxy *g);                 // 62b1:0000
void Preferences(Player *p);                // 6a9b:09a8
void Operations(Player *p, Empire *e, Galaxy *g);  // 6a9b:0000
void Communications(Player *p, Empire *e);             // 58b6:2481
void Advisors(Player *p, Empire *e);                   // 3ab7:0000

// Turn phases and other game modules.
int16 TradeArrivals(Player *p, Empire *e);             // 6e3f:1f14
void TurnReport(Player *p, Empire *e);                 // 462a:0b58
void Advisor(Player *p, Empire *e);                    // 6572:2459
void PayUpkeep(Player *p, Empire *e, Galaxy *g);       // 5e48:0b9f
void FoodMarket(Player *p, Empire *e, Galaxy *g);      // 512f:018b
void CovertOps(Player *p, Empire *e, Galaxy *g);       // 409c:0a6d
void Spending(Player *p, Empire *e, Galaxy *g);        // 1af6:07dd
void DoubleAgents(Player *p, Empire *e);               // 432f:006f
void Trade(Player *p, Empire *e);                      // 6e3f:2429
int16 SendMessage(int16 from, String name);            // 58b6:1ce9
class Streamable;
Streamable *ComposeMessage(Empire *e, int16 to);       // 58b6:1bee
int16 SysopMessages();                                 // 58b6:26ba
int16 MessagesEmpireDeleted(int16 id);                 // 58b6:25f9
void ClearMessages();                                  // 58b6:2a17
int16 ReadPublicMessages();                            // 58b6:2901
int16 ReadMessages(Player *p, Empire *e);              // 58b6:1eb3
int16 Attack(Player *p, Empire *e, char how);          // 5090:0121
void EndOfTurn(Player *p, Empire *e, Galaxy *g);       // 5f38:07ae
void Population(Player *p, Empire *e);                 // 5f38:0000
void CivilWar(Player *p, Empire *e);                   // 44e2:07cc
void Desertion(Player *p, Empire *e);                  // 44e2:0000
void TreatyMessages(Player *p, Empire *e);                  // 70d2:1ddf
void DiplomacyMenu(Player *p, Empire *e);               // 70d2:0000
void RegeneratePlanets(Galaxy *g);                     // 1c59:0855
extern Resource r_messages;                            // 62e8
extern Resource r_diplomacy;                           // b01e
extern Resource r_trading;                             // aa0a

extern Resource r_lottery;                             // b72a
void BuyTickets(Player *p, Empire *e, Galaxy *g);      // 55e7:0713
void DeleteEmpire(Empire *e);                          // 6924:057d
void Bank(Player *p, Empire *e, Galaxy *g);        // 3e41:0000
void Bulletins();                           // 5543:0918
void MoreHelp();                            // 5543:09a5
void ShowHelpTopic(String topic);           // 5543:02c5
void ShowHelpTopic(String prefix, String name);  // 5543:0366
void News(int16 n);                         // 5b6a:0228
void TimeCheck();                           // 1e7b:008d
void CheckRegistration();                   // 68a3:0124

#endif
