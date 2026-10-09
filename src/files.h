// File streams and record files of the SRCOMM library (segment 2144).
#ifndef SRE_FILES_H
#define SRE_FILES_H

#include "stream.h"
#include "sstring.h"
#include <stdio.h>

// Map a DOS-style relative path ("Data\Empire.II") to a host path in the
// game directory, matching names case-insensitively.
std::string HostPath(const char *dospath);
FILE *OpenHost(const char *dospath, const char *mode);
bool HostExists(const char *dospath);
int HostRemove(const char *dospath);
int HostRename(const char *from, const char *to);
int HostMkdir(const char *dospath);
int32 HostFileLength(const char *dospath);

// Output text file (vtable be43): an OStream whose bytes go to a file.
class OFStream : public OStream {
public:
    FILE *fp;
    int16 err;
    OFStream();
    OFStream(const char *name, int16 append, int16 binary = 1, int16 wrap = 0);
    ~OFStream();
    int16 open(const char *name, int16 append);
    void close();
    char PutRaw(char c);
    int16 fail() const { return fp == 0 || err != 0; }
};

// Input file (vtable be27): an IStream reading characters from a file.
// Text mode strips every CR and stops at ^Z, like Borland's read().
class IFStream : public IStream {
public:
    FILE *fp;
    int16 err;          // 1 at end of file
    int16 binary;
    IFStream();
    IFStream(const char *name, int16 binary);
    ~IFStream();
    int16 open(const char *name, int16 binary);
    void close();
    void Echo(char) {}
    int16 KeyReady() { return fp != 0 && err == 0; }
    char GetKey();
    int16 fail() const { return fp == 0 || err != 0; }
};

// 260c:0009: read one line (tabs become spaces, backspace edits).
void ReadLine(IStream &in, String &s);
OStream &operator<<(OStream &o, const String &s);

// Record file with XOR scrambling (2144:096f).
class DataFile {
public:
    char name[0x80];
    FILE *fp;
    int16 recsize;
    int16 err;
    int32 cachepos;
    uint8 key;
    uint8 mod;

    DataFile(const char *name, int16 recsize, uint8 key, uint8 mod);
    ~DataFile();
    int16 open(const char *name);
    void close();
    int16 read(int32 rec, void *buf, int16 count);
    int16 write(int32 rec, const void *buf, int16 count);
    int32 numrecs();
    int16 truncate(int32 nrecs);
    int16 erase();
    int16 error();
    void scramble(void *buf, int16 count);
};

#endif
