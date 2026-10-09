// RESOURCE.DAT configuration system (segments 2932 and 2a0c).
//
// Each line of RESOURCE.<node>, RESOURCE.DAT and RESOURCE.DEF is read
// with a small Lisp-like reader: the first token must be a symbol (the
// resource name) and the rest of the line is its value list.  A Resource
// object looks itself up lazily: first in the command line (/Name,
// /Name=1, /NoName), then in the loaded lines, where the first line
// loaded wins.
#ifndef SRE_RESOURCE_H
#define SRE_RESOURCE_H

#include "types.h"
#include "sstring.h"

enum AtomType { AT_ERROR = 0, AT_SYMBOL = 1, AT_QSYMBOL = 2, AT_STRING = 3,
                AT_NUMBER = 4, AT_BOOL = 5 };

struct Node {
    // An atom (string/number) or a cons cell (list).
    bool isCons;
    // atom
    String str;
    int32 num;
    int16 type;
    // cons
    Node *car;
    Node *cdr;
    bool shared;    // global constant (nil/#t/#f), never freed

    Node();
};

Node *ParseList(const String &s);       // 2a0c:0b1b
void FreeNode(Node *n);
Node *NodeCar(Node *list);              // 2a0c:005f (nil if not a cons)
Node *NodeCdr(Node *list);              // 2a0c:0082
Node *NodeNth(Node *list, int16 n);     // 2a0c:0148
int16 NodeLength(Node *list);           // 2a0c:0116
extern Node g_nil, g_true, g_false;

class Resource {
public:
    const char *name;
    Node *value;
    int16 resolved;
    int16 owned;

    Resource(const char *name, const char *def = 0);
    Resource(const char *name, int32 def);
    ~Resource();

    int32 Long();               // 2932:0225
    Node *List();               // 2932:02e0
    const char *Str();          // 2932:038b
    int16 Bool();               // 2932:0af0
    void Resolve();             // 2932:0467
    void SetDefault(const char *s);   // 2932:06c0
    void SetValue(Node *list);  // 2932:0847
};

// Load resource files (2932:08a0); called on first lookup.
void LoadResources();
// Load an additional resource file (flavors); entries are added in front.
void LoadResourceFile(const char *path);
// Is a plain (non-switch) argument equal to s (case-insensitive)?  2932:0c2c
int16 HasCommandArg(const char *s);

extern int g_argc;
extern char **g_argv;

#endif
