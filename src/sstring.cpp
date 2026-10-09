#include "sstring.h"
#include "brtl.h"
#include <string.h>
#include <stdlib.h>

String String::substr(int16 start, int16 count) const
{
    int16 len = length();
    if (len < start)
        start = len;
    if (count < 0)
        count = len;
    int16 end = start + count;
    if (len < end)
        end = len;
    String r;
    for (int16 i = start; i < end; i++)
        r += s_[i];
    return r;
}

void String::erase(int16 start, int16 count)
{
    int16 len = length();
    if (count == -1)
        count = len;
    if (len < start)
        start = len;
    if (start + count > len)
        count = len - start;
    if (count > 0)
        s_.erase(start, count);
}

void String::insert(int16 pos, const String &o)
{
    int16 len = length();
    if (len < pos)
        pos = len;
    s_.insert(pos, o.s_);
}

bool StrEqI(const char *a, const String &b)
{
    return b_stricmp(b.c_str(), a) == 0;
}

int16 String::find(const String &o, int16 from) const
{
    int16 last = o.length() + from - 1;
    if (last < 0)
        return 0;
    char lc = o.at(last);
    for (int16 i = last; i < length(); i++) {
        const void *p = memchr(s_.c_str() + i, (uint8)lc, length() - i);
        if (!p)
            break;
        i = (int16)((const char *)p - s_.c_str());
        if (at(i) != lc)
            continue;
        int16 j = 0, k = i - last;
        for (; j < last; j++, k++)
            if (o.at(j) != at(k))
                break;
        if (j >= last)
            return i - last;
    }
    return -1;
}

int16 String::findch(char c, int16 from) const
{
    int16 n = length();
    for (int16 i = from; i < n; i++)
        if (at(i) == c)
            return i;
    return -1;
}

bool StrEqI(const String &a, const String &b)
{
    return a.length() == b.length() && b_stricmp(b.c_str(), a.c_str()) == 0;
}

String NumStr(int32 v, int16 commas)
{
    String s;
    int16 neg = 0;
    if (v < 0) {
        neg = -1;
        v = -v;
    }
    if (v == 0) {
        s += '0';
    } else {
        while (v > 0) {
            s += (char)('0' + v % 10);
            v /= 10;
            if (s.length() % 4 == 3 && commas)
                s += ',';
        }
        if (commas && s.at(s.length() - 1) == ',')
            s.erase(s.length() - 1, 1);
        if (commas && s.length() == 5) {
            s.raw()[3] = s.at(4);
            s.erase(s.length() - 1, 1);
        }
    }
    if (neg)
        s += '-';
    std::string &r = s.raw();
    for (int16 i = 0; i < (int16)(r.size() / 2); i++) {
        char t = r[i];
        r[i] = r[r.size() - i - 1];
        r[r.size() - i - 1] = t;
    }
    return s;
}

int32 StrToLong(String s)
{
    for (int16 i = 0; i < s.length(); i++) {
        if (s.at(i) == ',') {
            s.erase(i, 1);
            i--;
        }
    }
    char *end;
    const char *p = s.c_str();
    long long w = strtoll(p, &end, 0);
    if (w > 0x7fffffffLL)
        w = 0x7fffffffLL;
    if (w < -0x80000000LL)
        w = -0x80000000LL;
    if (end > p)
        return (int32)w;
    return 0;
}

int32 RoundDigits(int32 v, int16 digits)
{
    if (v < 0)
        return -RoundDigits(-v, digits - 1);
    if (digits < 1 || digits > 9)
        return v;
    int32 lim = 1;
    int16 n;
    for (n = 0; n < digits; n++)
        lim *= 10;
    for (n = 0; v >= lim; n++) {
        int16 d = (int16)(v % 10);
        v /= 10;
        if (v < lim && d >= 5)
            v += 1;
    }
    for (; n > 0; n--)
        v *= 10;
    return v;
}
