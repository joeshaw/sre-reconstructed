// Data file integrity checks and encryption (segment 7420, 14f6).
#ifndef SRE_CRYPT_H
#define SRE_CRYPT_H

#include "game.h"

int32 Crc32(const void *data, uint16 len);
int32 Sum16(const void *data, uint16 len);
void Encrypt(void *data, uint16 len, const uint8 *salt, uint8 *keyout);
int16 Decrypt(void *data, uint16 len, const uint8 *salt, const uint8 *key);
Checksum MakeChecksum(void *data, uint16 len);          // 14f6:005c
int16 CheckChecksum(void *data, uint16 len, Checksum c); // 14f6:0147
int16 Unscramble(void *data, uint16 len, Checksum c);    // 14f6:01cb

#endif
