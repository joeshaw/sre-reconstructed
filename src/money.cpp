// Money (segment 28ec): a long kept within +-2,000,000,000.
#include "game.h"

static const int32 MONEY_MAX = 2000000000;
static const int32 MONEY_MIN = -2000000000;

// 28ec:003d: above the maximum clamps; below the minimum becomes 0.
void Money::set(int32 x)
{
    if (x > MONEY_MAX)
        v = MONEY_MAX;
    else if (x < MONEY_MIN)
        v = 0;
    else
        v = x;
}

// 28ec:00b3: saturating add, done in decreasing power-of-two steps.
void Money::add(int32 x)
{
    if (x < 0) {
        sub(-x);
        return;
    }
    int32 cur = v;
    for (int32 step = 0x10000000; step > 0; step /= 2) {
        while (cur < MONEY_MAX && x >= step && (int32)((uint32)cur + (uint32)step) > cur) {
            cur += step;
            x -= step;
        }
    }
    set(cur);
}

// 28ec:0199
void Money::sub(int32 x)
{
    if (x < 0) {
        add(-x);
        return;
    }
    int32 cur = v;
    for (int32 step = 0x10000000; step > 0; step /= 2) {
        while (cur > MONEY_MIN && x >= step && (int32)((uint32)cur - (uint32)step) < cur) {
            cur -= step;
            x -= step;
        }
    }
    set(cur);
}

// 28ec:029a
int16 MoneyOverflow(int32 a, int32 b)
{
    int32 s = (int32)((uint32)a + (uint32)b);
    if (a >= 0 && b >= 0 && s < 0)
        return 1;
    if (a >= 0 && b >= 0 && s > MONEY_MAX)
        return 1;
    if (a <= 0 && b <= 0 && s > 0)
        return 1;
    if (a <= 0 && b <= 0 && s < MONEY_MIN)
        return 1;
    return 0;
}
