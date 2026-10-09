// String class of the SRCOMM library (segment 273e).
#ifndef SRE_SSTRING_H
#define SRE_SSTRING_H

#include "types.h"
#include <string>

class String {
public:
    String() {}
    String(const char *s) : s_(s ? s : "") {}
    String(const String &o) : s_(o.s_) {}
    String(int16 n, char c) : s_(n > 0 ? n : 0, c) {}

    String &operator=(const String &o) { s_ = o.s_; return *this; }
    String &operator=(const char *s) { s_ = s ? s : ""; return *this; }
    String &operator+=(char c) { s_ += c; return *this; }
    String &operator+=(const char *s) { s_ += s; return *this; }
    String &operator+=(const String &o) { s_ += o.s_; return *this; }
    String operator+(const String &o) const { String r(*this); r.s_ += o.s_; return r; }
    String operator+(const char *o) const { String r(*this); r.s_ += o; return r; }

    int16 length() const { return (int16)s_.size(); }
    const char *c_str() const { return s_.c_str(); }
    operator const char *() const { return s_.c_str(); }

    // 273e:066d: character at i, or 0 past the end.
    char at(int16 i) const { return ((uint16)i < (uint16)s_.size()) ? s_[(uint16)i] : 0; }
    // 273e:0692: substring (start, count); count < 0 means to the end.
    String substr(int16 start, int16 count) const;
    // 273e:0738: erase count chars at start (-1 = to the end).
    void erase(int16 start, int16 count);
    // 273e:07cc: insert at position.
    void insert(int16 pos, const String &o);

    // 27eb:053c: index of o at or after from, or -1.
    int16 find(const String &o, int16 from = 0) const;
    // 27eb:0492: index of c at or after from, or -1.
    int16 findch(char c, int16 from = 0) const;

    bool operator==(const String &o) const { return s_ == o.s_; }
    bool operator!=(const String &o) const { return s_ != o.s_; }
    bool operator<(const String &o) const { return s_ < o.s_; }

    std::string &raw() { return s_; }
    const std::string &raw() const { return s_; }

private:
    std::string s_;
};

// 27eb:0425: case-insensitive comparison with a C string (stricmp == 0).
bool StrEqI(const char *a, const String &b);
// 27eb:03e5: case-insensitive equality of two Strings.
bool StrEqI(const String &a, const String &b);
// 27eb:0049: decimal text of v, with thousands separators if commas.
String NumStr(int32 v, int16 commas = 0);
// 27eb:0003 / 27eb:0026: String(int) / String(long).
inline String IntStr(int32 v) { return NumStr(v, 0); }
// 27eb:0255: parse a number, ignoring commas; 0 if not all digits.
int32 StrToLong(String s);
// 27eb:02d4: round v to the given number of decimal digits of
// precision (1..9).
int32 RoundDigits(int32 v, int16 digits);

#endif
