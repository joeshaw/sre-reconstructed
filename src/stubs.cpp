// Temporary stand-ins for modules not yet reconstructed.
#include "sre.h"

void Preferences(Player *) {}
void TimeCheck() {}
void ResearchInit() {}
void SysopMenu() {}
void TimeStart() {}
void InputDate(int32 *) {}
int16 MessagesEmpireDeleted(int16) { return 0; }
void ClearMessages() {}
int16 TradesEmpireDeleted(int16) { return 0; }
void ClearTrades() {}
Resource r_messages("Messages", (int32)1);     // 58b6 62e8
Resource r_diplomacy("Diplomacy", (int32)1);   // 70d2 b01e
void PlayGame(Player *) {}
void ReadMessages(Player *, Empire *) {}
int16 ReadPublicMessages() { return 0; }
void SendMessage(int16, String) {}
void Diplomacy(Player *, Empire *) {}
void DiplomacyEnd(Player *, Empire *) {}
