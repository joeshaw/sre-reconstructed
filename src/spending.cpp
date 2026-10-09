// Government spending: prices and the buy/sell menu (segment 1af6).
#include "sre.h"

// Prices.  The galaxy's w384 flag selects the formulas that grow with
// the empire's year and forces instead of only its net worth.

// 1af6:0006
int32 PriceSoldiers(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 1000 + 180;
    return (int32)(int16)((e->year << 1) + 180) + e->soldiers / 1800 + e->fighters / 12400 +
           NetWorth(e) / 5000;
}

// 1af6:00b8
int32 PriceFighters(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 800 + 240;
    return (int32)(int16)((int16)(e->year * 5) / 2 + 240) + e->soldiers / 7000 +
           e->fighters / 1690 + NetWorth(e) / 400;
}

// 1af6:0172
int32 PriceStations(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 700 + 320;
    return (int32)(int16)(e->year * 3 + 320) + e->soldiers / 7500 + e->fighters / 9000 +
           e->stations / 1460 + NetWorth(e) / 300;
}

// 1af6:024c
int32 PriceGenerals(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 300 + 670;
    return 14 * (int32)e->year + e->soldiers / 1790 + e->generals / 170 +
           TotalPlanets(e) / 10 + 670;
}

// 1af6:0307
int32 PriceCmdShip(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 100 + 8800;
    return e->soldiers / 850 + e->fighters / 950 + 5000;
}

// 1af6:0379
int32 PricePlanet(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 100 + TotalPlanets(e) + 13000;
    int32 k;
    if (e->year < 0)
        k = 3;
    else if (TotalPlanets(e) >= 300)
        k = 1;
    else
        k = 2;
    int32 x = TotalPlanets(e) + e->score / 2 + NetWorth(e) / 10 + (e->year < 0 ? 0 : e->year);
    return (97 * x + 50000) / k / 10;
}

// 1af6:0495
int32 PriceAgents(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 200 + 4300;
    return e->agents + 18 * (int32)e->year + 7 * TotalPlanets(e) + e->score / 10 + 2218;
}

// 1af6:0542
int32 PriceHeavyCruisers(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 600 + 1100;
    return 7 * (int32)e->year + e->fighters / 1220 + TotalPlanets(e) / 5 + e->stations / 2500 +
           NetWorth(e) / 150 + 1100;
}

// 1af6:0624
int32 PriceCarriers(Empire *e)
{
    if (g_galaxy.w384 == 0)
        return NetWorth(e) / 700 + 850;
    return 7 * (int32)e->year + TotalPlanets(e) / 10 + NetWorth(e) / 100 + 850;
}
