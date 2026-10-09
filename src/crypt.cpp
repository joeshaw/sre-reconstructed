// Data file integrity checks and encryption (segment 7420, 14f6:000f..01eb).
#include "crypt.h"
#include "brtl.h"
#include <string.h>

static uint32 g_crctab[0x100];      // DS:c11e
static int16 g_crcinit = 0;         // DS:c122

// 7420:0000
static void CrcInit()
{
    for (int16 i = 0; i < 0x100; i++) {
        uint32 v = (uint32)(int32)i;
        for (int16 k = 8; k > 0; k--) {
            if (v & 1)
                v = (v >> 1) ^ 0xedb88320UL;
            else
                v >>= 1;
        }
        g_crctab[i] = v;
    }
    g_crcinit = 1;
}

// 7420:009a
int32 Crc32(const void *data, uint16 len)
{
    if (!g_crcinit)
        CrcInit();
    const uint8 *p = (const uint8 *)data;
    uint32 crc = 0xffffffffUL;
    while (len--) {
        crc = ((crc >> 8) & 0x00ffffffUL) ^ g_crctab[(crc ^ *p++) & 0xff];
    }
    return (int32)crc;
}

// 7420:0130: a 16-bit running sum of sums (the routine computes more,
// but only this survives into the result).
int32 Sum16(const void *data, uint16 len)
{
    const uint8 *p = (const uint8 *)data;
    uint16 s1 = 0, s2 = 0;
    while (len--) {
        s1 += *p++;
        s2 += s1;
    }
    return (int32)(uint32)s2;
}

static uint8 Rol3(uint8 k)
{
    return (uint8)((k >> 5) | (uint8)(k << 3));
}

// 7420:0196: encrypt in place; returns the final key.
void Encrypt(void *data, uint16 len, const uint8 *salt, uint8 *keyout)
{
    uint8 k[8];
    uint8 *p = (uint8 *)data;
    memcpy(k, salt, 8);
    uint8 f = 0;
    for (uint16 i = 0; i < len; i++) {
        int16 j = i & 7;
        uint8 orig = p[i];
        p[i] ^= k[j];
        k[j] = Rol3(k[j]);
        k[j] ^= f;
        k[j] ^= orig;
        f += 7;
    }
    memcpy(keyout, k, 8);
}

// 7420:028c: decrypt in place; 1 if the final key matches.
int16 Decrypt(void *data, uint16 len, const uint8 *salt, const uint8 *key)
{
    uint8 k[8];
    uint8 *p = (uint8 *)data;
    memcpy(k, salt, 8);
    uint8 f = 0;
    for (uint16 i = 0; i < len; i++) {
        int16 j = i & 7;
        p[i] ^= k[j];
        k[j] = Rol3(k[j]);
        k[j] ^= f;
        k[j] ^= p[i];
        f += 7;
    }
    for (int16 i = 0; i < 8; i++)
        if (key[i] != k[i])
            return 0;
    return 1;
}

// 14f6:000f
void Checksum::clear()
{
    sum = 0;
    crc = 0;
    memset(salt, 0, 8);
    memset(key, 0, 8);
}

// 14f6:005c: checksum and encrypt (8 calls of rand()).
Checksum MakeChecksum(void *data, uint16 len)
{
    Checksum c;
    c.clear();
    int32 sum = Sum16(data, len);
    int32 crc = Crc32(data, len);
    uint8 salt[8], key[8];
    for (int16 i = 0; i < 8; i++)
        salt[i] = (uint8)(((int32)b_rand() << 8) / 0x8000);
    Encrypt(data, len, salt, key);
    c.sum = sum;
    c.crc = crc;
    memcpy(c.salt, salt, 8);
    memcpy(c.key, key, 8);
    return c;
}

// 14f6:0147: decrypt and verify both sums.
int16 CheckChecksum(void *data, uint16 len, Checksum c)
{
    if (!Decrypt(data, len, c.salt, c.key))
        return 0;
    int32 sum = Sum16(data, len);
    int32 crc = Crc32(data, len);
    if (c.sum != sum)
        return 0;
    if (c.crc != crc)
        return 0;
    return 1;
}

// 14f6:01cb: decrypt only.
int16 Unscramble(void *data, uint16 len, Checksum c)
{
    return Decrypt(data, len, c.salt, c.key);
}
