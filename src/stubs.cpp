// Temporary stand-ins for modules not yet reconstructed.
#include "sre.h"

void SysopMenu() {}
void TimeStart() {}
void InputDate(int32 *) {}
int16 MessagesEmpireDeleted(int16) { return 0; }
void ClearMessages() {}
int16 TradesEmpireDeleted(int16) { return 0; }
void ClearTrades() {}
Resource r_messages("Messages", (int32)1);     // 58b6 62e8
void ReadMessages(Player *, Empire *) {}
int16 ReadPublicMessages() { return 0; }
void SendMessage(int16, String) {}
Resource r_trading("Trading", (int32)1);       // 6e3f aa0a
void TradeArrivals(Player *, Empire *) {}
void Bank(Player *, Empire *, Galaxy *) {}
void Trade(Player *, Empire *) {}
int16 Attack(Player *, Empire *, char) { return 0; }
void Communications(Player *, Empire *) {}
