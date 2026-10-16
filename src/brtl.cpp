// Borland C++ 3.1 runtime behavior the game depends on.
#include "brtl.h"
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/time.h>
#include <unistd.h>

int16 b_stricmp(const char *a, const char *b)
{
    for (;;) {
        uint8 ca = (uint8)*a++;
        uint8 cb = (uint8)*b;
        if (ca == 0)
            return (int16)ca - (int16)cb;
        b++;
        if (ca == cb)
            continue;
        if (ca >= 'a' && ca <= 'z')
            ca -= 0x20;
        if (cb >= 'a' && cb <= 'z')
            cb -= 0x20;
        if (ca != cb)
            return (int16)ca - (int16)cb;
    }
}

int16 b_strnicmp(const char *a, const char *b, int16 n)
{
    uint8 ca = 0, cb = 0;
    while (n) {
        ca = (uint8)*a++;
        cb = (uint8)*b;
        if (ca == 0)
            break;
        b++;
        n--;
        if (ca == cb)
            continue;
        if (ca >= 'a' && ca <= 'z')
            ca -= 0x20;
        if (cb >= 'a' && cb <= 'z')
            cb -= 0x20;
        if (ca != cb)
            break;
        if (n == 0)
            return 0;
    }
    if (n == 0 && ca == cb)
        return 0;
    return (int16)ca - (int16)cb;
}

char *b_strupr(char *s)
{
    for (char *p = s; *p; p++)
        if (*p >= 'a' && *p <= 'z')
            *p -= 0x20;
    return s;
}

char *b_ltoa(int32 v, char *buf, int16 radix)
{
    char tmp[40];
    int n = 0;
    uint32 u;
    char *o = buf;
    if (radix == 10 && v < 0) {
        *o++ = '-';
        u = (uint32)(-(int64_t)v);
    } else {
        u = (uint32)v;
    }
    do {
        int d = (int)(u % (uint32)radix);
        tmp[n++] = (char)(d < 10 ? '0' + d : 'a' + d - 10);
        u /= (uint32)radix;
    } while (u);
    while (n)
        *o++ = tmp[--n];
    *o = 0;
    return buf;
}

int32 b_atol(const char *s)
{
    while (*s == ' ' || *s == '\t' || *s == '\n' || *s == '\r' || *s == '\v' || *s == '\f')
        s++;
    int neg = 0;
    if (*s == '-') {
        neg = 1;
        s++;
    } else if (*s == '+') {
        s++;
    }
    uint32 v = 0;
    while (*s >= '0' && *s <= '9')
        v = v * 10 + (uint32)(*s++ - '0');
    return neg ? -(int32)v : (int32)v;
}

int16 b_atoi(const char *s)
{
    return (int16)b_atol(s);
}

// ---------------------------------------------------------------------
static uint32 g_seed = 1;

int16 b_rand()
{
    g_seed = g_seed * 0x015A4E35UL + 1;
    return (int16)((g_seed >> 16) & 0x7fff);
}

void b_srand(uint16 seed)
{
    g_seed = seed;
}

uint32 b_randseed() { return g_seed; }
void b_setrandseed(uint32 s) { g_seed = s; }

// ---------------------------------------------------------------------
// Time.  timezone = 5 hours (EST), daylight = 1, as Borland's tzset()
// sets them when TZ is not defined.

static const int32 TIMEZONE = 18000;
static const int16 DAYLIGHT = 1;
static const int8 Days[12] = { 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 };
// DS:c72c; index 0 is the word before the cumulative table.
static const uint16 YDays[14] = { 7966, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365 };
#define YDAYS(i) YDays[(i) + 1]

// 1000:4cc8 __isDST(year - 1970, month (0 = use yday), yday, hour)
static int16 isDST(uint16 year, uint16 month, uint16 yday, uint8 hour)
{
    uint16 change;      // the day of the year the clocks change (a Sunday)
    if (month == 0) {
        uint16 day = yday;
        if (yday >= 59 && ((year + 70) & 3) == 0)
            day--;
        month = 0;
        while (YDAYS(month) <= day)
            month++;
    } else {
        yday += YDAYS(month - 1);
        if (month > 3 && ((year + 70) & 3) == 0)
            yday++;
    }
    if (month < 4)
        return 0;
    if (month > 10)
        return 0;
    if (month != 4 && month != 10)
        return 1;
    if ((int16)year > 16 && month == 4)
        change = YDAYS(month - 1) + 7;
    else
        change = YDAYS(month);
    if (((year + 0x7b2) & 3) != 0)
        change--;
    // Back to a Sunday: January 1, 1970 was a Thursday.
    uint16 days = (uint16)(((int16)(year + 1) >> 2) + change);
    uint16 epochday = (uint16)(0x16d * year + days + 4);
    change -= epochday % 7;
    if (month == 4) {
        if (yday > change)
            return 1;
        if (yday != change)
            return 0;
        return hour >= 2;
    }
    if (yday < change)
        return 1;
    if (yday != change)
        return 0;
    return hour <= 1;
}

int32 b_dostounix(const b_date *d, const b_dostime *t)
{
    int32 x = TIMEZONE + 315532800;
    int16 years = d->da_year - 1980;
    x += (int32)(years >> 2) * 126230400;
    x += (int32)(years & 3) * 31536000;
    if (years & 3)
        x += 86400;
    int16 days = 0;
    for (int16 i = d->da_mon - 1; i > 0;) {
        i--;
        days += Days[i];
    }
    days += d->da_day - 1;
    if (d->da_mon > 2 && (d->da_year & 3) == 0)
        days++;
    int16 hours = (int16)(days * 24 + t->ti_hour);
    if (DAYLIGHT && isDST((uint16)(d->da_year - 1970), 0, (uint16)days, t->ti_hour))
        hours--;
    x += (int32)hours * 3600;
    x += (int32)t->ti_min * 60 + t->ti_sec;
    return x;
}

void b_unixtodos(int32 time, b_date *d, b_dostime *t)
{
    time -= TIMEZONE + 315532800;
    t->ti_hund = 0;
    t->ti_sec = (uint8)(time % 60);
    time /= 60;
    t->ti_min = (uint8)(time % 60);
    time /= 60;
    d->da_year = (int16)(1980 + 4 * (int16)(time / 35064));
    time %= 35064;
    if (time >= 8784) {
        time -= 8784;
        d->da_year++;
        d->da_year += (int16)(time / 8760);
        time %= 8760;
    }
    if (DAYLIGHT && isDST((uint16)(d->da_year - 1970), 0, (uint16)(time / 24), (uint8)(time % 24)))
        time++;
    t->ti_hour = (uint8)(time % 24);
    time /= 24;
    time++;
    if ((d->da_year & 3) == 0) {
        if (time > 60) {
            time--;
        } else if (time == 60) {
            d->da_mon = 2;
            d->da_day = 29;
            return;
        }
    }
    d->da_mon = 0;
    while (Days[(int)d->da_mon] < time) {
        time -= Days[(int)d->da_mon];
        d->da_mon++;
    }
    d->da_mon++;
    d->da_day = (int8)time;
}

// ---------------------------------------------------------------------
// Clock source.

static int g_fixed = 0;
static int32 g_fixedbase;       // DOS local wall clock, as seconds since 1970 (UTC fields)
static int32 g_rate;            // 0 = frozen; 1 = real time
static uint32 g_ms0;
static int g_ms0set;

uint32 b_millis()
{
    struct timeval tv;
    gettimeofday(&tv, 0);
    uint32 ms = (uint32)(tv.tv_sec * 1000ULL + tv.tv_usec / 1000);
    if (!g_ms0set) {
        g_ms0 = ms;
        g_ms0set = 1;
    }
    return ms - g_ms0;
}

void b_setclock(int32 base, int32 rate)
{
    g_fixed = 1;
    g_fixedbase = base;
    g_rate = rate;
    b_millis();
}

static void wallclock(struct tm *out)
{
    if (g_fixed) {
        time_t t = (time_t)g_fixedbase;
        if (g_rate)
            t += (time_t)(b_millis() / 1000);
        gmtime_r(&t, out);
    } else {
        // The original only accepts dates from December 25, 1979 to
        // December 11, 1999 (SystemValid), so the host's date is moved
        // back by SRE_DATE_SHIFT years: 28 by default, which keeps the
        // calendar (weekdays and leap years) the same and works until
        // December 11, 2027.
        static int shift = -1;
        if (shift == -1) {
            const char *e = getenv("SRE_DATE_SHIFT");
            shift = e && *e ? atoi(e) : 28;
        }
        time_t t = time(0);
        localtime_r(&t, out);
        out->tm_year -= shift;
    }
}

void b_getdate(b_date *d)
{
    struct tm tm;
    wallclock(&tm);
    d->da_year = (int16)(tm.tm_year + 1900);
    d->da_mon = (int8)(tm.tm_mon + 1);
    d->da_day = (int8)tm.tm_mday;
}

void b_gettime(b_dostime *t)
{
    struct tm tm;
    wallclock(&tm);
    t->ti_hour = (uint8)tm.tm_hour;
    t->ti_min = (uint8)tm.tm_min;
    t->ti_sec = (uint8)tm.tm_sec;
    t->ti_hund = 0;
}

// 1000:0f03
int32 b_time(int32 *t)
{
    b_date d;
    b_dostime tm;
    b_getdate(&d);
    b_gettime(&tm);
    int32 v = b_dostounix(&d, &tm);
    if (t)
        *t = v;
    return v;
}

void b_delay(uint16 ms)
{
    if (g_fixed && g_rate == 0)
        return;
    usleep((useconds_t)ms * 1000);
}
