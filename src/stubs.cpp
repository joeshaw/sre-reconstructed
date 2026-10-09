// Temporary stand-ins for modules not yet reconstructed.
#include "sre.h"

void SysopMenu() {}
void TimeStart() {}
void InputDate(int32 *) {}
int16 TradesEmpireDeleted(int16) { return 0; }
void ClearTrades() {}
Resource r_trading("Trading", (int32)1);       // 6e3f aa0a
void TradeArrivals(Player *, Empire *) {}
void Trade(Player *, Empire *) {}
int16 Attack(Player *, Empire *, char) { return 0; }
