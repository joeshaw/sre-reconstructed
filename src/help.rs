// Help topics and bulletins from Text/Topics.HLP and Text/SRTopics.HLP
// (segment 5543).  A topic is the text between "#key <name>" and "#end";
// "\n" in the text is a line break and a backslash at the end of a line
// stops that line.

use crate::prelude::*;

const TOPICS_FILE: &[u8] = b"Text/Topics.HLP"; // DS:4644
const SRTOPICS_FILE: &[u8] = b"Text/SRTopics.HLP"; // DS:4648

// 5543:0000: show a topic from a file; the number of topics shown.
fn show_topic(g: &mut Sre, file: &[u8], topic: &[u8]) -> i16 {
    let f = ifstream(g, file, false);
    let mut n: i16 = 0;
    let mut c: u8;
    let fail = |g: &mut Sre| ifstream_fail(g, f);
    while !fail(g) {
        c = in_char(g, f);
        if c != b'#' {
            while !fail(g) && c != 0 && c != b'\n' {
                c = in_char(g, f);
            }
            c = b'?';
        }
        while !fail(g) && c != 0 && c != b'#' {
            c = in_char(g, f);
            if c != b'#' {
                while !fail(g) && c != 0 && c != b'\n' {
                    c = in_char(g, f);
                }
                c = b'?';
            }
        }
        if fail(g) {
            break;
        }
        let mut head = Str::new();
        if c == b'#' {
            head.push(c);
            head.extend(in_line(g, f));
        }
        if c == b'#' && b_strnicmp(&head, b"#EOF", 4) == 0 {
            break;
        }
        if c != b'#' {
            continue;
        }
        if b_strnicmp(&head, b"#key ", 5) != 0 {
            continue;
        }
        let mut i: i16 = 5;
        while ctype_signed(at(&head, i)) & CT_SP != 0 && at(&head, i) != 0 {
            i += 1;
        }
        if !str_eq_i2(topic, &head[(i as usize).min(head.len())..]) {
            continue;
        }
        set_maxlen(g, f, 0xff);
        loop {
            let line = in_line(g, f);
            if b_strnicmp(&line, b"#end", 4) == 0 {
                break;
            }
            let mut j: i16 = 0;
            while at(&line, j) != 0 {
                if at(&line, j) == b'\\' && at(&line, j + 1) == 0 {
                    break;
                }
                if at(&line, j) == b'\\' && at(&line, j + 1) == b'n' {
                    kout!(g; b'\n');
                    j += 1;
                } else {
                    kout!(g; at(&line, j));
                }
                j += 1;
            }
            if !(b_strnicmp(&line, b"#end", 4) != 0 && !fail(g)) {
                break;
            }
        }
        set_maxlen(g, f, 0);
        n += 1;
    }
    ifstream_done(g, f);
    n
}

// 5543:02c5
pub fn show_help_topic(g: &mut Sre, topic: &[u8]) {
    if show_topic(g, TOPICS_FILE, topic) != 0 {
        return;
    }
    if show_topic(g, SRTOPICS_FILE, topic) != 0 {
        return;
    }
    kout!(g; "Text for ", topic, " not found.\n");
}

// 5543:0366
pub fn show_help_topic2(g: &mut Sre, prefix: &[u8], name: &[u8]) {
    let t = cat(&[prefix, b"-", name]);
    show_help_topic(g, &t);
}

// 5543:0583: the names of the topics "<prefix>-<name>", newest first.
fn list_topics(g: &mut Sre, file: &[u8], prefix: &[u8], names: &mut Vec<Str>) {
    let f = ifstream(g, file, false);
    let key = cat(&[b"#key ", prefix]);
    let len = key.len();
    while !ifstream_fail(g, f) {
        let mut c = in_char(g, f);
        let mut head = Str::new();
        if c == b'#' {
            head.push(c);
            head.extend(in_line(g, f));
        } else {
            while c != 0 && c != b'\n' {
                c = in_char(g, f);
            }
            c = 0;
        }
        if c == b'#' && b_strnicmp(&head, b"#EOF", 4) == 0 {
            break;
        }
        if c == b'#' && b_strnicmp(&head, &key, len as i16) == 0 && at(&head, len as i16) == b'-' {
            let name: Str = head[len + 1..].to_vec();
            let found = names.iter().any(|n| str_eq_i2(&name, n));
            if !found {
                names.insert(0, name);
            }
        }
    }
    ifstream_done(g, f);
}

// 5543:06d5: a menu of the topics with a prefix.
fn topic_menu(g: &mut Sre, m: &mut Menu, prefix: &[u8]) {
    let mut names: Vec<Str> = Vec::new();
    list_topics(g, TOPICS_FILE, prefix, &mut names);
    list_topics(g, SRTOPICS_FILE, prefix, &mut names);
    let mut n: i16 = 0;
    for name in names.iter() {
        n += 1;
        m.add(g, &int_str(n as i32), 0, n);
        m.find(n).unwrap().add_column_str(name);
    }
    if n < 20 {
        m.columns = 1;
    }
    loop {
        m.show(g);
        kout!(g; cn(g), "(0=Relist; Enter=Quit)  ");
        set_defval(g, KIN, -1);
        set_maxval(g, KIN, n as i32);
        let v = in_i16(g, KIN);
        if v == -1 {
            break;
        }
        if v < 1 || v >= n {
            continue;
        }
        let name = names[(v - 1) as usize].clone();
        kout!(g; cn(g));
        show_help_topic2(g, prefix, &name);
        pause_prompt(g);
    }
}

// 5543:0918
pub fn bulletins(g: &mut Sre) {
    let mut m = Menu::new(g, b"Bulletins", b"Available Bulletins", [0xf, 1, 1, 0xf, 0xf, 7, 7, 7, 7, 7]);
    m.columns = 10;
    topic_menu(g, &mut m, b"bulletin");
}

// 5543:09a5
pub fn more_help(g: &mut Sre) {
    let mut m = Menu::new(g, b"Help", b"Extra Help", [0xf, 4, 4, 0xf, 0xf, 7, 7, 7, 7, 7]);
    m.columns = 10;
    topic_menu(g, &mut m, b"help");
}
