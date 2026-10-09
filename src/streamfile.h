// Files of streamable objects (segment 1629).
//
// A stream file is a sequence of records, each a 0x50-byte header (data
// size, checksum of the class name, the class name, checksum of the
// data) followed by the encrypted data.  Reading creates objects by
// class name through a registry.  A StreamFile reads the old file and
// writes the objects to keep into a temporary file, which replaces the
// original when it is closed.
#ifndef SRE_STREAMFILE_H
#define SRE_STREAMFILE_H

#include "types.h"
#include "sstring.h"

// Base of the streamable classes (vtable 3836).  The data of an object
// is Size() - 2 bytes (the 2 being the original's vtable pointer).
class Streamable {
public:
    virtual ~Streamable() {}
    virtual const char *ClassName() = 0;            // [04]
    virtual int16 Size() = 0;                       // [08] 1629:000c
    virtual int16 Expired() { return 1; }           // [0c] 1629:0016
    virtual void OnEmpireDeleted(int16) {}          // [10] 1629:0020
    virtual uint8 *Data() = 0;
};

typedef Streamable *(*CreateFn)();
struct ClassEntry {
    const char *name;
    CreateFn create;
    ClassEntry *next;
};
// Add a class to the registry (static constructors, DS:43f0).
void RegisterClass(ClassEntry *e, const char *name, CreateFn fn);
// 1629:004e
Streamable *CreateObject(const char *name);

// A file handle (1629:0b4d).
class RawFile {
public:
    int fd;
    String name;
    int16 err;
    RawFile(const char *name = 0, int16 mode = 0);
    ~RawFile();
    void open(const String &name, int16 mode);      // 1629:0c06
    void close();                                   // 1629:0d2c
    int16 read(void *buf, int16 n);                 // 1629:0d73
    int16 write(const void *buf, int16 n);          // 1629:0dd2
    void skip(int16 n);                             // 1629:0e9b
};

// File open modes, as Borland's O_ flags.
enum { FM_READ = 1, FM_WRITE = 2, FM_APPEND = 0x800 };

Streamable *ReadObject(RawFile *f);                 // 1629:0102
void WriteObject(RawFile *f, Streamable *o);        // 1629:059c

class StreamFile {
public:
    RawFile in;         // +00
    RawFile out;        // +0c
    String path;        // +18
    String temp;        // +20
    explicit StreamFile(const char *name);          // 1629:07c7
    ~StreamFile();                                  // 1629:095d
    Streamable *Read() { return ReadObject(&in); }
    void Write(Streamable *o) { WriteObject(&out, o); }
    // Write o back unless it has expired (the pattern after each read).
    void Keep(Streamable *o);
};

// 1629:0ac6: append an object to a stream file.
void AppendObject(const char *name, Streamable *o);

#endif
