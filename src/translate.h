// Output translation (segment 155b): "Output.TranslationFile" lets a sysop
// replace words in all game text, and a few colour/bracket resources.
#ifndef SRE_TRANSLATE_H
#define SRE_TRANSLATE_H

#include "sstring.h"
#include "stream.h"

// 155b:027c: translate a string; the result is valid until the next call.
const char *Translate(const char *s);
// 155b:00b2: length of the translated string.
int16 DisplayLen(const String &s);
// 155b:0313: read the translation file and the Output.* resources.
void LoadTranslation();

extern Color c_bracket;     // DS:6786 Output.Bracket.Color
extern Color c_bold;        // DS:678a Output.Bold.Color
extern Color c_number;      // DS:677e Output.Number.Color
extern Color c_empire;      // DS:678e Output.Empire.Color

#endif
