// Borland C++ 3.1 runtime behaviour the game depends on.
#ifndef SRE_BRTL_H
#define SRE_BRTL_H

#include "types.h"

// String helpers (1000:459d stricmp, 1000:462e strnicmp, 1000:4788 strupr).
int16 b_stricmp(const char *a, const char *b);
int16 b_strnicmp(const char *a, const char *b, int16 n);
char *b_strupr(char *s);
char *b_ltoa(int32 v, char *buf, int16 radix);
int32 b_atol(const char *s);
int16 b_atoi(const char *s);

// rand()/srand() (1000:0b9d, 1000:0b8c).
int16 b_rand();
void b_srand(uint16 seed);
uint32 b_randseed();
void b_setrandseed(uint32 s);

// DOS date/time structures and conversions (1000:47ac dostounix,
// 1000:48d6 unixtodos, 1000:0f03 time) with Borland's default
// TZ = EST5EDT and its US daylight rule.
struct b_date { int16 da_year; int8 da_day; int8 da_mon; };
struct b_dostime { uint8 ti_min; uint8 ti_hour; uint8 ti_hund; uint8 ti_sec; };
int32 b_dostounix(const b_date *d, const b_dostime *t);
void b_unixtodos(int32 time, b_date *d, b_dostime *t);
int32 b_time(int32 *t);
void b_getdate(b_date *d);
void b_gettime(b_dostime *t);

// The "DOS clock".  Normally the host's local time; tests may pin it.
void b_setclock(int32 fixed_dos_seconds_since_1970, int32 rate);
// Milliseconds elapsed (for delay() and timeouts).
uint32 b_millis();
void b_delay(uint16 ms);

#endif
