// Dates and times (segment 206a).
#ifndef SRE_TIMEUTIL_H
#define SRE_TIMEUTIL_H

#include "types.h"
#include "stream.h"

extern int32 g_now;             // time at start-up
extern int32 g_today;           // midnight today
extern char g_timesep;
extern int16 g_time24;

void InitTime();
int16 TimeSecond(int32 t);
int16 TimeMinute(int32 t);
int16 TimeHour(int32 t);
int16 TimeDay(int32 t);
int16 TimeMonth(int32 t);
int16 TimeYear(int32 t);
int32 Midnight(int32 t);
int32 MakeDate(int16 year, int16 month, int16 day);
const char *TimeStr(int32 t);
OStream &PutTime(OStream &o, int32 t);
// Midnight of the latest representable date: "no end date".
inline int32 NeverDate() { return Midnight(0x7fffffff); }

#endif
