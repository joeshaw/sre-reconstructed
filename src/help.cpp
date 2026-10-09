// Help topics and bulletins from Text/Topics.HLP and Text/SRTopics.HLP
// (segment 5543).  A topic is the text between "#key <name>" and "#end";
// "\n" in the text is a line break and a backslash at the end of a line
// stops that line.
#include "sre.h"
#include <list>

static const char *TOPICS_FILE = "Text/Topics.HLP";       // DS:4644
static const char *SRTOPICS_FILE = "Text/SRTopics.HLP";   // DS:4648

// 5543:0000: show a topic from a file; the number of topics shown.
static int16 ShowTopic(const char *file, const String &topic)
{
    IFStream f(file, 0);
    int16 n = 0;
    char c = 0;
    while (!f.fail()) {
        f >> c;
        if (c != '#') {
            while (!f.fail() && c != 0 && c != '\n')
                f >> c;
            c = '?';
        }
        while (!f.fail() && c != 0 && c != '#') {
            f >> c;
            if (c != '#') {
                while (!f.fail() && c != 0 && c != '\n')
                    f >> c;
                c = '?';
            }
        }
        if (f.fail())
            break;
        char head[0x100];
        if (c == '#') {
            head[0] = c;
            f >> (head + 1);
        }
        if (c == '#' && b_strnicmp(head, "#EOF", 4) == 0)
            break;
        if (c != '#')
            continue;
        if (b_strnicmp(head, "#key ", 5) != 0)
            continue;
        int16 i = 5;
        while ((ctype_signed(head[i]) & CT_SP) && head[i] != 0)
            i++;
        if (!StrEqI(topic, String(head + i)))
            continue;
        f >> maxlen(0xff);
        char line[0x100];
        do {
            f >> line;
            if (b_strnicmp(line, "#end", 4) == 0)
                break;
            for (int16 j = 0; line[j]; j++) {
                if (line[j] == '\\' && line[j + 1] == 0)
                    break;
                if (line[j] == '\\' && line[j + 1] == 'n') {
                    kout << '\n';
                    j++;
                } else {
                    kout << line[j];
                }
            }
        } while (b_strnicmp(line, "#end", 4) != 0 && !f.fail());
        f >> maxlen(0);
        n++;
    }
    return n;
}

// 5543:02c5
void ShowHelpTopic(String topic)
{
    if (ShowTopic(TOPICS_FILE, topic))
        return;
    if (ShowTopic(SRTOPICS_FILE, topic))
        return;
    kout << "Text for " << topic << " not found.\n";
}

// 5543:0366
void ShowHelpTopic(String prefix, String name)
{
    ShowHelpTopic(prefix + "-" + name);
}

// 5543:0583: the names of the topics "<prefix>-<name>", newest first.
static void ListTopics(const char *file, const char *prefix, std::list<String> &names)
{
    IFStream f(file, 0);
    char key[0xa0];
    strcpy(key, "#key ");
    strcat(key, prefix);
    int16 len = (int16)strlen(key);
    while (!f.fail()) {
        char c;
        char head[0x100];
        f >> c;
        if (c == '#') {
            head[0] = c;
            f >> (head + 1);
        } else {
            while (c != 0 && c != '\n')
                f >> c;
            c = 0;
        }
        if (c == '#' && b_strnicmp(head, "#EOF", 4) == 0)
            break;
        if (c == '#' && b_strnicmp(head, key, len) == 0 && head[len] == '-') {
            String name(head + len + 1);
            bool found = false;
            for (std::list<String>::iterator it = names.begin(); it != names.end(); ++it)
                if (StrEqI(name, *it))
                    found = true;
            if (!found)
                names.push_front(name);
        }
    }
}

// 5543:06d5: a menu of the topics with a prefix.
static void TopicMenu(Menu &m, const char *prefix)
{
    std::list<String> names;
    ListTopics(TOPICS_FILE, prefix, names);
    ListTopics(SRTOPICS_FILE, prefix, names);
    int16 n = 0;
    for (std::list<String>::iterator it = names.begin(); it != names.end(); ++it) {
        n++;
        m.Add(IntStr(n), 0, n);
        m.Find(n)->AddColumn(it->c_str());
    }
    if (n < 20)
        m.columns = 1;
    for (;;) {
        m.Show();
        kout << c_normal << "(0=Relist; Enter=Quit)  ";
        int16 v;
        kin >> defval(-1) >> maxval(n) >> v;
        if (v == -1)
            break;
        if (v < 1 || v >= n)
            continue;
        std::list<String>::iterator it = names.begin();
        while (--v != 0)
            ++it;
        kout << c_normal;
        ShowHelpTopic(String(prefix), *it);
        Pause();
    }
}

// 5543:0918
void Bulletins()
{
    Menu m("Bulletins", "Available Bulletins", 0xf, 1, 1, 0xf, 0xf, 7, 7, 7, 7, 7);
    m.columns = 10;
    TopicMenu(m, "bulletin");
}

// 5543:09a5
void MoreHelp()
{
    Menu m("Help", "Extra Help", 0xf, 4, 4, 0xf, 0xf, 7, 7, 7, 7, 7);
    m.columns = 10;
    TopicMenu(m, "help");
}
