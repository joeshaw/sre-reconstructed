// Reports: messages to empires, kept in stream files (segment 69d7).
#ifndef SRE_REPORT_H
#define SRE_REPORT_H

#include "stream.h"
#include "streamfile.h"
#include "game.h"

extern const char *REPORT_FILE;     // DS:91e3 "Data/Report"
extern int16 g_newreports;          // DS:91d2

#pragma pack(push, 1)
struct ReportData {
    int32 created;      // 02
    int32 expires;      // 06
    int16 count;        // 0a
    int16 to[25];       // 0c recipients
    uint16 len;         // 3e
    char text[0x200];   // 40
};
#pragma pack(pop)

// Class "Report" (vtable 9269, 0x240 bytes).
class Report : public Streamable {
public:
    ReportData d;
    Report();                                       // 69d7:022f
    const char *ClassName();                        // 69d7:005f
    int16 Size() { return (int16)(d.len + 0x41); }  // 69d7:006d: as long as the text
    int16 Expired();                                // 69d7:007e
    void OnEmpireDeleted(int16 id);                 // 69d7:01c9
    uint8 *Data() { return (uint8 *)&d; }
    int16 HasRecipient(int16 id);                   // 69d7:018d
    int16 AddRecipient(int16 id);                   // 69d7:0571
    int16 RemoveRecipient(int16 id);                // 69d7:05c5
    void AddAll();                                  // 69d7:0624
    void ClearRecipients() { d.count = 0; }         // 69d7:066e
    void ShowHeader(int16 n, int16 showtime, int16 c1, int16 c2);  // 69d7:0362
    void Show();                                    // 69d7:0454
};

// An output stream that collects a report and files it when destroyed
// (vtable 924d, 0x264 bytes).
class ReportOut : public OStream {
public:
    Report rep;             // +20
    const char *file;       // +260
    explicit ReportOut(const char *file = 0);       // 69d7:089d (no recipient)
    ReportOut(int16 id, const char *file);          // 69d7:0920
    ~ReportOut();                                   // 69d7:09eb
    char PutRaw(char c);                            // 69d7:0a7b
    void EmitColor();                               // 69d7:0acd
    // The report's date.
    void SetTime(int32 t) { rep.d.created = t; }
};

int16 ShowReports(int16 id, const char *file, int16 showtime, int16 c1, int16 c2);  // 69d7:067c
int16 ShowNewReports(Player *p, Empire *e);         // 69d7:0777
int16 ReportsEmpireDeleted(int16 id, const char *file);  // 69d7:07bb
void ClearStreamFile(const char *file);             // 69d7:0b6f
char *ReportBuffer();                               // 69d7:0000

#endif
