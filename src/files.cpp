// File streams and record files (segment 2144).
#include "files.h"
#include "brtl.h"
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>
#include <errno.h>

// ---------------------------------------------------------------------
// Path mapping.  The game uses DOS paths relative to its directory with
// either separator; on the host each component is matched without
// regard to case, and new names are created as given.

std::string HostPath(const char *dospath)
{
    std::string out = ".";
    const char *p = dospath;
    if (p[0] && p[1] == ':')
        p += 2;
    while (*p == '\\' || *p == '/')
        p++;
    std::string comp;
    for (;;) {
        char c = *p;
        if (c == '\\' || c == '/' || c == 0) {
            if (!comp.empty() && comp != ".") {
                std::string found = comp;
                for (size_t k = 0; k < found.size(); k++)
                    if (found[k] >= 'a' && found[k] <= 'z')
                        found[k] = (char)(found[k] - 32);
                DIR *d = opendir(out.c_str());
                if (d) {
                    struct dirent *e;
                    while ((e = readdir(d)) != 0) {
                        if (b_stricmp(e->d_name, comp.c_str()) == 0) {
                            found = e->d_name;
                            break;
                        }
                    }
                    closedir(d);
                }
                out += "/";
                out += found;
            }
            comp.clear();
            if (c == 0)
                break;
        } else {
            comp += c;
        }
        p++;
    }
    return out;
}

FILE *OpenHost(const char *dospath, const char *mode)
{
    return fopen(HostPath(dospath).c_str(), mode);
}

bool HostExists(const char *dospath)
{
    struct stat st;
    return stat(HostPath(dospath).c_str(), &st) == 0;
}

int HostRemove(const char *dospath)
{
    return remove(HostPath(dospath).c_str());
}

int HostRename(const char *from, const char *to)
{
    return rename(HostPath(from).c_str(), HostPath(to).c_str());
}

int HostMkdir(const char *dospath)
{
    return mkdir(HostPath(dospath).c_str(), 0777);
}

int32 HostFileLength(const char *dospath)
{
    struct stat st;
    if (stat(HostPath(dospath).c_str(), &st) != 0)
        return -1;
    return (int32)st.st_size;
}

static void FileError(const char *fmt, const char *name);

// ---------------------------------------------------------------------
// OFStream

OFStream::OFStream() : OStream(0), fp(0), err(0) {}

OFStream::OFStream(const char *name, int16 append, int16 binary, int16 wrap)
    : OStream(wrap), fp(0), err(0)
{
    (void)binary;
    open(name, append);
}

OFStream::~OFStream()
{
    close();
}

// 2144:0234
int16 OFStream::open(const char *name, int16 append)
{
    close();
    fp = OpenHost(name, append ? "ab" : "wb");
    if (!fp) {
        err = 2;
        FileError("File '%s' not openable for output.\n", name);
        return -1;
    }
    err = 0;
    return 0;
}

void OFStream::close()
{
    if (fp)
        fclose(fp);
    fp = 0;
    err = 0;
}

// 2144:03fa
char OFStream::PutRaw(char c)
{
    if (!fp)
        return 0;
    if (fputc((uint8)c, fp) == EOF)
        return 0;
    return c;
}

// ---------------------------------------------------------------------
// IFStream

IFStream::IFStream() : fp(0), err(0), binary(0) {}

IFStream::IFStream(const char *name, int16 bin) : fp(0), err(0), binary(0)
{
    open(name, bin);
}

IFStream::~IFStream()
{
    close();
}

// 2144:06c4
int16 IFStream::open(const char *name, int16 bin)
{
    close();
    binary = bin;
    fp = OpenHost(name, "rb");
    if (!fp) {
        err = 2;
        return -1;
    }
    err = 0;
    return 0;
}

void IFStream::close()
{
    if (fp)
        fclose(fp);
    fp = 0;
    err = 0;
}

// 2144:07e5 (with Borland text-mode read(): CRs dropped, ^Z = EOF)
char IFStream::GetKey()
{
    if (!fp)
        return 0;
    for (;;) {
        int c = fgetc(fp);
        if (c == EOF) {
            err = 1;
            return 0;
        }
        if (!binary) {
            if (c == '\r')
                continue;
            if (c == 0x1a) {
                err = 1;
                return 0;
            }
        }
        return (char)c;
    }
}

// 260c:0009
void ReadLine(IStream &in, String &s)
{
    s = "";
    for (;;) {
        char ch;
        in >> ch;
        int16 c = (uint8)ch;
        if (c == 9)
            c = ' ';
        if (c == 8)
            s.erase(s.length() - 1, 1);
        else if (c >= 0x20 && c <= 0xfe && c != 0x7f)
            s += (char)c;
        if (c == '\n' || c == '\r' || c == 0)
            break;
    }
}

// 260c:009f
OStream &operator<<(OStream &o, const String &s)
{
    return o << s.c_str();
}

// ---------------------------------------------------------------------
// DataFile


static FileErrorFn g_fileerror = 0;

static void FileError(const char *fmt, const char *name)
{
    // 2144:1054: expand %s and pass to the error logger.
    char out[0x104];
    int16 o = 0;
    for (int16 i = 0; fmt[i]; i++) {
        if (fmt[i] == '%' && fmt[i + 1] == 's') {
            strcpy(out + o, name);
            o += (int16)strlen(name);
            i++;
        } else {
            out[o++] = fmt[i];
            out[o] = 0;
        }
    }
    out[o] = 0;
    if (g_fileerror)
        g_fileerror(out);
}

void SetFileErrorHandler(FileErrorFn fn)
{
    g_fileerror = fn;
}

DataFile::DataFile(const char *n, int16 rs, uint8 k, uint8 m)
{
    name[0] = 0;
    cachepos = -1;
    mod = m < 1 ? 1 : m;
    key = k;
    err = 0;
    recsize = rs;
    fp = 0;
    if (n && n[0])
        open(n);
}

DataFile::~DataFile()
{
    close();
    name[0] = 0;
}

// 2144:0a56
int16 DataFile::open(const char *n)
{
    if (fp)
        close();
    std::string hp = HostPath(n);
    fp = fopen(hp.c_str(), "r+b");
    if (!fp)
        fp = fopen(hp.c_str(), "w+b");
    if (!fp) {
        err = 2;
        FileError("File '%s' not openable.\n", n);
    }
    strncpy(name, n, 0x7f);
    name[0x7f] = 0;
    return 0;
}

void DataFile::close()
{
    if (!fp)
        return;
    fclose(fp);
    fp = 0;
    err = 0;
    name[0] = 0;
}

// 2144:0b5c: symmetric scrambling of count records.
void DataFile::scramble(void *buf, int16 count)
{
    uint8 *p = (uint8 *)buf + (int32)recsize * count;
    if (key == 0 && mod == 1)
        return;
    for (int16 r = count - 1; r >= 0; r--)
        for (int16 j = recsize - 1; j >= 0; j--) {
            p--;
            *p ^= (uint8)(key + j % mod);
        }
}

// 2144:0f6d
int32 DataFile::numrecs()
{
    if (!fp)
        return 0;
    long cur = ftell(fp);
    fseek(fp, 0, SEEK_END);
    long len = ftell(fp);
    fseek(fp, cur, SEEK_SET);
    return (int32)(len / recsize);
}

// 2144:0be7
int16 DataFile::read(int32 rec, void *buf, int16 count)
{
    if (!fp)
        return 0;
    if (fseek(fp, (long)rec * recsize, SEEK_SET) != 0) {
        err = 6;
        return 0;
    }
    int16 want = (int16)(recsize * count);
    size_t got = fread(buf, 1, (uint16)want, fp);
    if ((int16)got < want) {
        err = 1;
        return 0;
    }
    scramble(buf, count);
    return 1;
}

// 2144:0d28
int16 DataFile::write(int32 rec, const void *buf, int16 count)
{
    if (!fp)
        return 0;
    if (fseek(fp, (long)rec * recsize, SEEK_SET) != 0) {
        err = 6;
        return 0;
    }
    int16 want = (int16)(recsize * count);
    uint8 *tmp = new uint8[(uint16)want];
    memcpy(tmp, buf, (uint16)want);
    scramble(tmp, count);
    size_t put = fwrite(tmp, 1, (uint16)want, fp);
    fflush(fp);
    delete[] tmp;
    if ((int16)put < want) {
        FileError("Error writing to file '%s', disk full?\n", name);
        err = 1;
        return 0;
    }
    return 1;
}

int16 DataFile::truncate(int32 nrecs)
{
    if (!fp)
        return 0;
    fflush(fp);
    return ftruncate(fileno(fp), (off_t)nrecs * recsize) == 0;
}

// 2144:0fcf: delete the file and reopen it empty.
int16 DataFile::erase()
{
    char n[0x80];
    strcpy(n, name);
    close();
    int r = HostRemove(n);
    open(n);
    return r != -1;
}

int16 DataFile::error()
{
    int16 e = err;
    err = 0;
    return e;
}
