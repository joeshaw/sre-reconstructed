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
void GalaxyInit(Galaxy *g);
void GalaxyReset(Galaxy *g);
void GalaxyOpen();
void GalaxyClose();
void ScoreList(Galaxy *g, int32 mask, OStream *o);
int32 EmpireMask();
int32 ProtectedMask();
int16 GalaxyPlayable(Galaxy *g);
void ShowMasters(Galaxy *g);
extern Resource r_inactivetimeout, r_failedcovert, r_attackvictim, r_pirateraids, r_civilwar;

// empire (48cc, 1c59)
void EmpireInit(Empire *e);
int32 TotalPlanets(Empire *e);
int32 NetWorth(Empire *e);
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
void UpdateD1b0(Empire *e);
int32 GovCapacity(Empire *e);
int32 GovCapacity6(Empire *e);
const char *InsurgencyName(Empire *e);
const char *PlanetName(int8 type);
void InsurgencyUp(Empire *e);
void InsurgencyDown(Empire *e);
void ClearSpending(uint8 *s);
void SetTreaty(Treaty &t, int16 type);

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
void Bulletins();                           // 5543:0918
void MoreHelp();                            // 5543:09a5
void News(int16 n);                         // 5b6a:0228
void TimeCheck();                           // 1e7b:008d
void CheckRegistration();                   // 68a3:0124

#endif
