// Files of streamable objects (segment 1629).
#include "sre.h"
#include "streamfile.h"
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>

static ClassEntry *g_classes;           // DS:43f0
static const char *g_streamdir = "";    // DS:43ec

void RegisterClass(ClassEntry *e, const char *name, CreateFn fn)
{
    e->name = name;
    e->create = fn;
    e->next = g_classes;
    g_classes = e;
}

// 1629:004e
Streamable *CreateObject(const char *name)
{
    for (ClassEntry *e = g_classes; e; e = e->next) {
        if (b_stricmp(e->name, name) == 0)
            return e->create ? e->create() : 0;
    }
    PutTime(g_errlog, g_now) << " Tried to create object of type " << name << ".\n";
    return 0;
}

// Borland's errno values for the cases the game reports.
static int16 DosErrno(int e)
{
    if (e == ENOENT)
        return 2;
    if (e == EMFILE || e == ENFILE)
        return 4;
    if (e == EACCES || e == EPERM)
        return 5;
    return 6;
}

// 1629:0b4d
RawFile::RawFile(const char *n, int16 mode) : fd(-1), err(1)
{
    if (n)
        open(String(n), mode);
}

// 1629:0bc2
RawFile::~RawFile()
{
    close();
}

// 1629:0c06: open (always creating), binary.
void RawFile::open(const String &n, int16 mode)
{
    int flags = O_CREAT;
    if (mode & FM_READ)
        flags |= O_RDONLY;
    else if (mode & FM_WRITE)
        flags |= O_WRONLY;
    if (mode & FM_APPEND)
        flags |= O_APPEND;
    fd = ::open(HostPath(n.c_str()).c_str(), flags, 0666);
    err = 0;
    if (fd == -1) {
        err = DosErrno(errno);
        if (err == 2)
            PutTime(g_errlog, g_now) << " File '" << n << "' not openable.\n";
        if (err == 4)
            PutTime(g_errlog, g_now) << " Too many files open.  Try increasing FILES= in CONFIG.SYS.\n";
    }
    if ((mode & FM_WRITE) && fd != -1)
        lseek(fd, 0, SEEK_END);
    name = n;
}

// 1629:0d2c
void RawFile::close()
{
    if (fd == -1)
        return;
    ::close(fd);
    fd = -1;
    err = 0;
    name = "";
}

// 1629:0d73
int16 RawFile::read(void *buf, int16 n)
{
    if (fd == -1)
        return 0;
    ssize_t r = ::read(fd, buf, (uint16)n);
    if (r == -1) {
        err = DosErrno(errno);
        return 0;
    }
    if (r < (uint16)n) {
        err = 1;
        return 0;
    }
    return 1;
}

// 1629:0dd2
int16 RawFile::write(const void *buf, int16 n)
{
    if (fd == -1 || !buf)
        return 0;
    ssize_t r = ::write(fd, buf, (uint16)n);
    if (r == -1) {
        err = DosErrno(errno);
        return 0;
    }
    if (r < (uint16)n) {
        PutTime(g_errlog, g_now) << " Error writing to file ' " << name << "', disk full?\n";
        err = 1;
        return 0;
    }
    return 1;
}

// 1629:0e9b
void RawFile::skip(int16 n)
{
    if (fd != -1)
        lseek(fd, n, SEEK_CUR);
}

#pragma pack(push, 1)
struct RecordHeader {
    int16 size;
    Checksum namesum;
    char name[0x1e];
    Checksum datasum;
};
#pragma pack(pop)
static_assert(sizeof(RecordHeader) == 0x50, "RecordHeader");

// 1629:0102: the next object of the file, or 0 at the end.
Streamable *ReadObject(RawFile *f)
{
    for (;;) {
        RecordHeader h;
        h.namesum.clear();
        h.datasum.clear();
        h.name[0] = 0;
        h.size = 0;
        if (!f->read(&h, 0x50))
            return 0;
        if (h.size < 0) {
            PutTime(g_errlog, g_now) << " File `" << f->name
                                     << "' negative packet size, skipping record.\n";
            return 0;
        }
        if (!CheckChecksum(h.name, 0x1e, h.namesum)) {
            PutTime(g_errlog, g_now) << " File `" << f->name
                                     << "' header corrupted, skipping record.\n";
            f->skip(h.size);
            continue;
        }
        h.name[0x1d] = 0;
        Streamable *o = CreateObject(h.name);
        if (!o) {
            PutTime(g_errlog, g_now) << " Trying to read from file `" << f->name << "', couldn't match ";
            g_errlog << "class header {" << h.name << "}.  It may mean `out of memory', ";
            g_errlog << "but it's probably Amit's fault.\n";
            f->skip(h.size);
            continue;
        }
        memset(o->Data(), 0, o->Size() - 2);
        if (!f->read(o->Data(), h.size)) {
            PutTime(g_errlog, g_now) << " Trying to read from file `" << f->name
                                     << "', couldn't read bytes.\n";
            f->close();
        }
        if (!CheckChecksum(o->Data(), h.size, h.datasum)) {
            PutTime(g_errlog, g_now) << " Data security check failed.  File `" << f->name
                                     << "' could be corrupt.\n";
            delete o;
            continue;
        }
        return o;
    }
}

// 1629:059c
void WriteObject(RawFile *f, Streamable *o)
{
    if (!o) {
        PutTime(g_errlog, g_now) << " Received null pointer in streamfile output to `"
                                 << f->name << "'.\n";
        return;
    }
    RecordHeader h;
    h.namesum.clear();
    h.datasum.clear();
    h.size = (int16)(o->Size() - 2);
    strncpy(h.name, o->ClassName(), 0x1d);
    h.name[0x1d] = 0;
    h.datasum = MakeChecksum(o->Data(), h.size);
    h.namesum = MakeChecksum(h.name, 0x1e);
    f->write(&h, 0x50);
    f->write(o->Data(), h.size);
    Unscramble(h.name, 0x1e, h.namesum);
    Unscramble(o->Data(), h.size, h.datasum);
}

// 1629:07c7
StreamFile::StreamFile(const char *name)
{
    path = String(g_streamdir) + name;
    in.open(path, FM_READ);
    temp = String(name) + ".$$$";
    HostRemove(temp.c_str());
    out.open(temp, FM_WRITE | FM_APPEND);
}

// 1629:095d
StreamFile::~StreamFile()
{
    if (out.err == 0) {
        in.close();
        out.close();
        HostRemove(path.c_str());
        HostRename(temp.c_str(), path.c_str());
    } else {
        in.close();
        out.close();
        HostRemove(temp.c_str());
        PutTime(g_errlog, g_now) << " Error writing to " << path << ".\n";
    }
}

void StreamFile::Keep(Streamable *o)
{
    if (o && !o->Expired())
        Write(o);
}

// 1629:0ac6
void AppendObject(const char *name, Streamable *o)
{
    String p = String(g_streamdir) + name;
    RawFile f(p.c_str(), FM_WRITE | FM_APPEND);
    WriteObject(&f, o);
}
