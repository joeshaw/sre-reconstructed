// Battle reports and attack news (segment 3f62).
#ifndef SRE_BATTLE_H
#define SRE_BATTLE_H

#include "report.h"

extern const char *BATTLE_FILE;     // DS:0d5c "Data/Battles"

// A report about attacks (vtable 0f63).
class BattleReport : public ReportOut {
public:
    BattleReport() : ReportOut(BATTLE_FILE) {}                      // 3f62:0000
    explicit BattleReport(int16 id) : ReportOut(id, BATTLE_FILE) {} // 3f62:0047
};

void AttackMessage(OStream &o, const char *attacker, int16 severity);       // 3f62:00cb
int16 ShowBattleReports(Player *p, Empire *e);                              // 3f62:01c1
void ClearBattleReports();                                                  // 3f62:01f5
void AttackNews(Empire *a, Empire *d, int16 success, int16 severity);       // 3f62:0209

#endif
