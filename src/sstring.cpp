#include "sstring.h"
#include "brtl.h"

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
