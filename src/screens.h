// Screens, prompts and text files (segments 43e0 and 7389).
#ifndef SRE_SCREENS_H
#define SRE_SCREENS_H

#include "sstring.h"
#include "stream.h"

extern String g_version;        // DS:baec "0.994b"
extern String g_copyright;      // DS:baf4
extern String g_flavordir;      // DS:d130 "flavors\<flavor>\"
extern char g_answer;           // DS:c92c last yes/no answer

void ClearScreen();                                     // 43e0:0000
void Centered(int16 fg, const String &s);               // 43e0:0056
void TitleScreen();                                     // 43e0:00da
void StatusBar(const char *who, const char *what, char c);  // 43e0:042c (sysop screen only)
// 43e0:08a4: show base.ANS (if ANSI and IBM), else base.MSG or base.TXT.
int16 ShowFileBase(String base, int16 paged);
// 43e0:0aa1: look for a screen in the flavor directory, then in dir
// (default "Screens/").
int16 ShowScreen(const char *dir, String name, int16 paged);
// 43e0:0b60: "question?  [y/N]"; def 0 = no, 1 = yes, 2 = must answer.
int16 YesNo(int16 def, const char *question);
// 43e0:0f44: a line of n box characters in color c.
void HLine(Color c, int16 n);

// 7389:063a: write a line with "^\BF" color codes.
void PutCoded(const String &s, OStream &o);
// 7389:07c6: display a text file (each byte XORed with key).
int16 DisplayFile(const char *name, uint8 key);

#endif
