// Dates and times (segment 206a).  Times are DOS-local seconds since
// 1970 as returned by Borland's time().
#include "timeutil.h"
#include "brtl.h"
#include "devices.h"
#include <string.h>
#include <stdio.h>

int32 g_now;            // DS:d35c
int32 g_today;          // DS:bbb9
char g_timesep = ':';   // DS:bb82 Output.Time.Separator
int16 g_time24 = 0;     // DS:bb83 Output.Time.24Hour

// 206a:0bce (static constructors)
void InitTime()
{
    g_now = b_time(0);
    g_today = Midnight(g_now);
}

static void Split(int32 t, b_date *d, b_dostime *tm)
{
    b_unixtodos(t, d, tm);
}

int16 TimeSecond(int32 t) { b_date d; b_dostime m; Split(t, &d, &m); return m.ti_sec; }    // 206a:008c
int16 TimeMinute(int32 t) { b_date d; b_dostime m; Split(t, &d, &m); return m.ti_min; }    // 206a:00a8
int16 TimeHour(int32 t) { b_date d; b_dostime m; Split(t, &d, &m); return m.ti_hour; }     // 206a:00c2
int16 TimeDay(int32 t) { b_date d; b_dostime m; Split(t, &d, &m); return d.da_day; }       // 206a:00de
int16 TimeMonth(int32 t) { b_date d; b_dostime m; Split(t, &d, &m); return d.da_mon; }     // 206a:00f9
int16 TimeYear(int32 t) { b_date d; b_dostime m; Split(t, &d, &m); return d.da_year; }     // 206a:0114

// 206a:01b1
int32 Midnight(int32 t)
{
    b_date d;
    b_dostime m;
    b_unixtodos(t, &d, &m);
    m.ti_hund = 0;
    m.ti_hour = 0;
    m.ti_min = 0;
    m.ti_sec = 0;
    return b_dostounix(&d, &m);
}

// 206a:0251
int32 MakeDate(int16 year, int16 month, int16 day)
{
    b_date d;
    b_dostime m;
    m.ti_hund = 0;
    m.ti_hour = 0;
    m.ti_min = 0;
    m.ti_sec = 0;
    d.da_year = year;
    d.da_mon = (int8)month;
    d.da_day = (int8)day;
    return b_dostounix(&d, &m);
}

static const char *const MONTHS[13] = { "", "Jan", "Feb", "Mar", "Apr", "May", "Jun",
                                        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec" };

// 206a:02e7: "8-Mar-1995 12:00pm"; the time part only if not midnight.
const char *TimeStr(int32 t)
{
    static char buf[0x30];      // DS:d334
    b_date d;
    b_dostime m;
    char num[16];
    b_unixtodos(t, &d, &m);
    buf[0] = 0;
    b_ltoa((int8)d.da_day, num, 10);
    strcat(buf, num);
    strcat(buf, "-");
    strcat(buf, MONTHS[(int8)d.da_mon]);
    strcat(buf, "-");
    b_ltoa(d.da_year, num, 10);
    strcat(buf, num);
    if (m.ti_hour == 0 && m.ti_min == 0 && m.ti_hund == 0 && m.ti_sec == 0)
        return buf;
    strcat(buf, " ");
    int16 h = m.ti_hour;
    if (!g_time24) {
        if (h >= 12)
            h -= 12;
        if (h == 0)
            h = 12;
    }
    b_ltoa(h, num, 10);
    strcat(buf, num);
    int16 n = (int16)strlen(buf);
    buf[n++] = g_timesep;
    buf[n++] = (char)(m.ti_min / 10 + '0');
    buf[n++] = (char)(m.ti_min % 10 + '0');
    if (!g_time24) {
        buf[n++] = (m.ti_hour < 12) ? 'a' : 'p';
        buf[n++] = 'm';
    }
    buf[n] = 0;
    return buf;
}

// 206a:02bd
OStream &PutTime(OStream &o, int32 t)
{
    return o << TimeStr(t);
}
