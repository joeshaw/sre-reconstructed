// Messages between empires (segment 58b6).
#ifndef SRE_MESSAGES_H
#define SRE_MESSAGES_H

#include "sre.h"
#include "streamfile.h"

extern const char *MESSAGE_CLASS;   // DS:6335 "Message"

enum { MAXLINES = 99, LINELEN = 0x50, QUOTE = '\xff' };

#pragma pack(push, 1)
struct MessageData {
    int16 w0;               // 00
    int16 w2;               // 02
    int16 from;             // 04 sending empire, -1 = sysop/non-player
    int32 created;          // 06
    int32 expires;          // 0a
    char name[0x20];        // 0e sender as shown
    char to[0x20];          // 2e recipient letters
    int16 list[25];         // 4e recipients (public: those who read it)
    int16 count;            // 80
    uint8 flags;            // 82 1 anonymous, 2 public
    int16 nlines;           // 83
    char lines[MAXLINES][LINELEN];  // 85
};
#pragma pack(pop)
static_assert(sizeof(MessageData) == 0x1f75, "MessageData");

enum { MF_ANON = 1, MF_PUBLIC = 2 };

// Class "Message" (vtable 676a).
class Message : public Streamable {
public:
    MessageData d;
    Message() { Clear(); }                                  // 58b6:047e
    const char *ClassName() { return MESSAGE_CLASS; }       // 58b6:01a4
    int16 Size() { return (int16)(d.nlines * LINELEN + 0x87); }  // 58b6:01bb
    uint8 *Data() { return (uint8 *)&d; }
    int16 Expired();                                        // 58b6:01d5
    void OnEmpireDeleted(int16 id);                         // 58b6:01ec
    void Clear();                                           // 58b6:0387
    int16 Receives(int16 id);                               // 58b6:0205
    void MarkRead(int16 id);                                // 58b6:029a
    int16 AddressedTo(int16 id);                            // 58b6:04f6
    void Header(int16 n, int16 c1, int16 c2);               // 58b6:054b
    void ShowInfo(int16 c);                                 // 58b6:0868
    void ShowLines(int16 c1, int16 c2, int16 c3, int16 c4); // 58b6:0961
    void LineNumber(int16 c);                               // 58b6:0aa4
    void Edit(char quote, int16 c);                         // 58b6:0b54
    void AddRecipient(int16 id);                            // 58b6:13ce
    void RemoveRecipient(int16 id);                         // 58b6:1425
    void SortRecipients();                                  // 58b6:1487
    void Address(int16 from, int16 to);                     // 58b6:156a
    int16 PrepareReply(int16 me);                           // 58b6:15db
    int16 ChooseRecipients(int16 from);                     // 58b6:181e
    int16 Compose(String sender, char quote);               // 58b6:195c
};

#endif
