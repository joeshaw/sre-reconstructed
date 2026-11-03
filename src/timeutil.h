// Dates and times (segment 206a).
#ifndef SRE_TIMEUTIL_H
#define SRE_TIMEUTIL_H

#include "types.h"
#include "stream.h"

// 206a: a time, as DOS-local seconds since 1970 (Borland's time()).
// Like Money, code uses it like a long.  The original also has a
// constructor that sets it to the current time (206a:0056), operator-
// for the seconds between two times (206a:012c), operator+ for a time
// plus seconds (206a:013f) and operator<< for streams (206a:02bd); the
// reconstruction sets times explicitly and does the arithmetic on longs.
struct Time {
    int32 v;
    Time &operator=(int32 t) { v = t; return *this; }
    operator int32() const { return v; }
    int16 second() const;       // 206a:008c
    int16 minute() const;       // 206a:00a8
    int16 hour() const;         // 206a:00c2
    int16 day() const;          // 206a:00de
    int16 month() const;        // 206a:00f9
    int16 year() const;         // 206a:0114
};

extern Time g_now;              // time at start-up
extern Time g_today;            // midnight today
extern char g_timesep;
extern int16 g_time24;

void InitTime();
Time Midnight(int32 t);
Time MakeDate(int16 year, int16 month, int16 day);
void InputDate(Time *t);
const char *TimeStr(int32 t);
OStream &PutTime(OStream &o, int32 t);
// Midnight of the latest representable date: "no end date".
inline int32 NeverDate() { return Midnight(0x7fffffff); }

#endif
