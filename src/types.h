// Fixed-width types matching Borland C++ 3.1 (16-bit int, 32-bit long).
// The game's arithmetic depends on 16-bit wraparound in places, so the
// reconstruction uses explicit widths everywhere and is compiled with
// -fwrapv.
#ifndef SRE_TYPES_H
#define SRE_TYPES_H

#include <stdint.h>

typedef int8_t int8;
typedef uint8_t uint8;
typedef int16_t int16;
typedef uint16_t uint16;
typedef int32_t int32;
typedef uint32_t uint32;

// Borland's _ctype[] classification bits.
enum {
    CT_SP = 1, CT_DIG = 2, CT_UPP = 4, CT_LOW = 8,
    CT_HEX = 16, CT_CTL = 32, CT_PUN = 64
};

// _ctype lookup with an unsigned index (the usual case).
int ctype_of(uint8 c);
// _ctype lookup with a sign-extended char index, as some original code
// does; for c >= 0x80 this reads the bytes in front of the table.
int ctype_signed(int8 c);

inline int b_isdigit(uint8 c) { return ctype_of(c) & CT_DIG; }
inline int b_isalpha(uint8 c) { return ctype_of(c) & (CT_UPP | CT_LOW); }
inline int b_isalnum(uint8 c) { return ctype_of(c) & (CT_DIG | CT_UPP | CT_LOW); }
inline int b_isspace(uint8 c) { return ctype_of(c) & CT_SP; }
inline int b_isupper(uint8 c) { return ctype_of(c) & CT_UPP; }
inline int b_islower(uint8 c) { return ctype_of(c) & CT_LOW; }
inline int b_iscntrl(uint8 c) { return ctype_of(c) & CT_CTL; }
inline int b_ispunct(uint8 c) { return ctype_of(c) & CT_PUN; }
// Borland toupper/tolower (1000:0fa1, 1000:0f75): ASCII only.
inline int b_toupper(int c) { return (c != -1 && b_islower((uint8)c)) ? (uint8)c - 32 : (c == -1 ? -1 : (uint8)c); }
inline int b_tolower(int c) { return (c != -1 && b_isupper((uint8)c)) ? (uint8)c + 32 : (c == -1 ? -1 : (uint8)c); }

#endif
