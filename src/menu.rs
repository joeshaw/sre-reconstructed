// Menus of the SRCOMM library (segment 1922).
//
// A menu has a name (used to look up "Menu.<name>.*" resources), a title,
// ten colour bytes (high nibble background, low nibble foreground) and up
// to 128 items.  Items may carry up to five extra columns (strings or
// numbers), used for tables like the rankings.

use crate::resource::{as_atom, node_nth, Resource, AT_NUMBER};
use crate::sre::Sre;
use crate::sstring::{at, cat, int_str, substr, Str};
use crate::stream::*;
use crate::translate::display_len;
use crate::types::b_toupper;
use crate::kout;

const BAR: u8 = 0xc4;

pub struct MenuChars {
    pub itemopen: u8, // DS:61e0 '<' '>'
    pub itemclose: u8,
    pub titleopen: u8, // DS:61e2 '[' ']'
    pub titleclose: u8,
    pub defopen: u8, // DS:61e4 '[' ']'
    pub defclose: u8,
}

impl Default for MenuChars {
    fn default() -> Self {
        MenuChars { itemopen: b'<', itemclose: b'>', titleopen: b'[', titleclose: b']', defopen: b'[', defclose: b']' }
    }
}

// Titles starting with '\' are shown literally (without the '\').
fn put_title(g: &mut Sre, t: &[u8]) {
    if at(t, 0) == b'\\' {
        let s = substr(t, 1, 0xff);
        put_untranslated(g, KOUT, &s);
    } else {
        put_str(g, KOUT, t);
    }
}

// A callback object (an object whose first virtual function is called).
pub type MenuHook = Box<dyn FnMut(&mut Sre) -> i16>;

pub struct MenuItem {
    pub menuname: Str,    // +00
    pub text: Str,        // +08
    pub key: u8,          // +10
    pub id: i16,          // +11
    pub colors: [u8; 8],  // +13 bracket, key, text, column 1..5
    pub ncols: i16,       // +1b
    pub coltype: [u8; 5], // +1d 's' or 'n'
    pub colstr: [Option<Str>; 5], // +22 (for 's')
    pub colnum: [i32; 5], // +22 (for 'n')
}

pub struct Menu {
    pub name: Str,                // +00
    pub title: Str,               // +08
    pub columns: i16,             // +12
    pub prompt: Option<MenuHook>, // +14
    pub display: Option<MenuHook>, // +18
    pub colors: [u8; 10],         // +1c title, frame, then the items' (bracket, key, text, columns 1..5)
    pub items: Vec<MenuItem>,     // +26
}

fn menu_res(name: &[u8], rest: &[u8]) -> Str {
    cat(&[b"Menu.", name, rest])
}

impl Menu {
    // 1922:001f
    pub fn new(g: &mut Sre, name: &[u8], title: &[u8], c: [u8; 10]) -> Menu {
        let mut m = Menu {
            name: name.to_vec(),
            title: title.to_vec(),
            columns: 1,
            prompt: None,
            display: None,
            colors: [0; 10],
            items: Vec::new(),
        };
        {
            let mut r = Resource::num(&menu_res(name, b".Title"), 1);
            let l = r.list(&mut g.resdb);
            if let Some(a) = as_atom(&l) {
                if a.num() == 0 {
                    m.title = a.str().to_vec();
                }
            }
        }
        let mut cc: [i16; 10] = c.map(|x| x as i16);
        {
            let mut r = Resource::text(&menu_res(name, b".Colors"), Some(b""));
            let l = r.list(&mut g.resdb);
            if l.as_deref().map_or(false, |n| n.is_cons()) {
                for (i, ci) in cc.iter_mut().enumerate() {
                    let it = node_nth(&l, i as i16);
                    if let Some(a) = as_atom(&it) {
                        if a.typ() == AT_NUMBER {
                            *ci = a.num() as i16;
                        }
                    }
                }
            }
        }
        for i in 0..10 {
            m.colors[i] = cc[i] as u8;
        }
        m
    }

    pub fn count(&self) -> i16 {
        self.items.len() as i16
    }

    // 1922:04dc
    pub fn add(&mut self, g: &mut Sre, text: &[u8], key: u8, id: i16) -> i16 {
        if self.count() > 0x80 {
            return 0;
        }
        let mut text = text.to_vec();
        {
            let rn = cat(&[b"Menu.", &self.name, b".", &int_str(id as i32), b".Name"]);
            let mut r = Resource::num(&rn, 1);
            let l = r.list(&mut g.resdb);
            if let Some(a) = as_atom(&l) {
                if a.num() == 0 {
                    text = a.str().to_vec();
                }
            }
        }
        let mut it = MenuItem::new(g, &self.name, &text, key, id);
        for i in 0..8 {
            it.colors[i] = self.colors[2 + i];
        }
        it.load_colors(g);
        self.items.push(it);
        self.count() - 1
    }

    // 1922:085f
    pub fn find(&mut self, id: i16) -> Option<&mut MenuItem> {
        self.items.iter_mut().find(|it| it.id == id)
    }

    // 1922:0966
    pub fn run(&mut self, g: &mut Sre, show: bool, def: i16) -> i16 {
        {
            let mut r = Resource::num(&menu_res(&self.name, b".Columns"), 0);
            let l = r.list(&mut g.resdb);
            if let Some(a) = as_atom(&l) {
                if a.num() > 0 {
                    self.columns = a.num() as i16;
                }
            }
        }
        if show {
            self.show(g);
        } else {
            let (to, tc) = (g.menuchars.titleopen, g.menuchars.titleclose);
            kout!(g; b'\n', byte_color(self.colors[1]), b"\xc4\xc4", to, byte_color(self.colors[0]));
            let t = self.title.clone();
            put_title(g, &t);
            let cn = g.dev.c_normal;
            kout!(g; byte_color(self.colors[1]), tc, b"\xc4\xc4", cn, b'\n');
        }
        if self.prompt(g) && def != -1 {
            for i in 0..self.items.len() {
                if self.items[i].id != def {
                    continue;
                }
                if (self.items[i].key as i8) <= 0x20 {
                    continue;
                }
                if self.colors[3] != 0 {
                    let (cn, ci) = (g.dev.c_normal, g.dev.c_input);
                    let (d0, d1) = (g.menuchars.defopen, g.menuchars.defclose);
                    kout!(g; cn, d0, "Enter=", byte_color(self.colors[3]), self.items[i].key, cn, d1, "  ", ci);
                }
                break;
            }
        }
        loop {
            let mut c = b_toupper(in_char(g, KIN));
            if c == b'?' {
                let cn = g.dev.c_normal;
                kout!(g; "Display Menu", cn, b'\n');
                self.show(g);
                self.prompt(g);
            } else {
                for i in 0..self.items.len() {
                    if self.items[i].key != c {
                        continue;
                    }
                    if !self.items[i].text.is_empty() {
                        let t = self.items[i].text.clone();
                        kout!(g; t);
                    } else {
                        kout!(g; c);
                    }
                    let cn = g.dev.c_normal;
                    kout!(g; cn, b'\n');
                    return self.items[i].id;
                }
                if c == b'Q' || c == b'0' {
                    c = 0x1b;
                }
                if c == b'\r' {
                    if def != -1 {
                        if let Some(it) = self.find(def) {
                            if !it.text.is_empty() {
                                let t = it.text.clone();
                                kout!(g; t);
                            }
                        }
                    }
                    let cn = g.dev.c_normal;
                    kout!(g; cn, b'\n');
                    return def;
                }
            }
            if c == 0x1b {
                let cn = g.dev.c_normal;
                kout!(g; "Quit", cn, b'\n');
                return -1;
            }
        }
    }

    // 1922:0eca
    pub fn show(&mut self, g: &mut Sre) {
        let mut maxtot: i16 = 0;
        let mut maxw: i16 = 0;
        let mut colw = [0i16; 5];
        for i in 0..self.items.len() {
            let t = self.items[i].text.clone();
            let mut w = display_len(g, &t);
            let mut hascols = false;
            for k in 0..5 {
                let cw = self.items[i].col_width(k as i16 + 1);
                if cw > 0 {
                    hascols = true;
                }
                if cw > colw[k] {
                    colw[k] = cw;
                }
            }
            if hascols {
                w += 2;
            }
            if w > maxw {
                maxw = w;
            }
        }
        for i in 0..self.items.len() {
            let t = self.items[i].total_width(g, maxw, &colw);
            if t > maxtot {
                maxtot = t;
            }
        }
        let count = self.count();
        let mut ncols: i16 = (0x4d / (maxtot as i32 + 2)) as i16;
        if ncols < 1 {
            ncols = 1;
        }
        if self.columns < ncols {
            ncols = self.columns;
        }
        if count < ncols {
            ncols = count;
        }
        let title = self.title.clone();
        let mut width = display_len(g, &title) + 5;
        if at(&title, 0) == b'\\' {
            width -= 1;
        }
        if (maxtot as i32 + 2) * ncols as i32 - 2 > width as i32 {
            width = ((maxtot as i32 + 2) * ncols as i32 - 2) as i16;
        }
        kout!(g; b'\n');
        if display_len(g, &title) > 0 {
            let mut tl = display_len(g, &title) + 2;
            if at(&title, 0) == b'\\' {
                tl -= 1;
            }
            kout!(g; byte_color(self.colors[1]));
            let mut k: i16 = 0;
            while k < (width - tl) / 2 {
                kout!(g; BAR);
                k += 1;
            }
            let (to, tc) = (g.menuchars.titleopen, g.menuchars.titleclose);
            kout!(g; to, byte_color(self.colors[0]));
            put_title(g, &title);
            kout!(g; byte_color(self.colors[1]), tc);
            while k < width - tl {
                kout!(g; BAR);
                k += 1;
            }
            kout!(g; b'\n');
        } else {
            kout!(g; byte_color(self.colors[1]));
            for _ in 0..width.max(0) {
                kout!(g; BAR);
            }
            kout!(g; b'\n');
        }
        if let Some(mut d) = self.display.take() {
            d(g);
            self.display = Some(d);
        }
        if ncols > 0 {
            let rows = ((count as i32 + ncols as i32 - 1) / ncols as i32) as i16;
            for r in 0..rows {
                if key_ready(g, KIN) {
                    break;
                }
                let mut i = r;
                while i < count {
                    let last = !(count > i + rows);
                    self.items[i as usize].draw(g, last, maxw, &colw);
                    if !last {
                        kout!(g; "  ");
                    }
                    i += rows;
                }
            }
        }
        if !key_ready(g, KIN) {
            kout!(g; byte_color(self.colors[1]));
            for _ in 0..width.max(0) {
                kout!(g; BAR);
            }
            kout!(g; b'\n');
        }
        let cn = g.dev.c_normal;
        kout!(g; cn);
    }

    // 1922:1467
    pub fn prompt(&mut self, g: &mut Sre) -> bool {
        let r = match self.prompt.take() {
            Some(mut p) => {
                let r = p(g);
                self.prompt = Some(p);
                r
            }
            None => 1,
        };
        if r != 0 {
            let cn = g.dev.c_normal;
            kout!(g; cn, "Which one?  ");
        }
        kout!(g; col(15));
        r != 0
    }
}

impl MenuItem {
    // 1922:14e5
    pub fn new(g: &mut Sre, menuname: &[u8], text: &[u8], key: u8, id: i16) -> MenuItem {
        let mut it = MenuItem {
            menuname: menuname.to_vec(),
            text: text.to_vec(),
            key,
            id,
            colors: [7; 8],
            ncols: 0,
            coltype: [0; 5],
            colstr: Default::default(),
            colnum: [0; 5],
        };
        it.load_colors(g);
        it
    }

    // 1922:16b2
    pub fn load_colors(&mut self, g: &mut Sre) {
        let mut c: [i16; 8] = self.colors.map(|x| x as i16);
        let rn = cat(&[b"Menu.", &self.menuname, b".", &int_str(self.id as i32), b".Colors"]);
        let mut r = Resource::text(&rn, Some(b""));
        let l = r.list(&mut g.resdb);
        if l.as_deref().map_or(false, |n| n.is_cons()) {
            for (i, ci) in c.iter_mut().enumerate() {
                let it = node_nth(&l, i as i16);
                if let Some(a) = as_atom(&it) {
                    if a.typ() == AT_NUMBER {
                        *ci = a.num() as i16;
                    }
                }
            }
        }
        for i in 0..8 {
            self.colors[i] = c[i] as u8;
        }
    }

    // 1922:160a
    pub fn add_column_str(&mut self, s: &[u8]) -> bool {
        if self.ncols >= 5 {
            return false;
        }
        let n = self.ncols as usize;
        self.coltype[n] = b's';
        self.colstr[n] = Some(s.to_vec());
        self.ncols += 1;
        true
    }

    // 1922:165e
    pub fn add_column_num(&mut self, v: i32) -> bool {
        if self.ncols >= 5 {
            return false;
        }
        let n = self.ncols as usize;
        self.coltype[n] = b'n';
        self.colnum[n] = v;
        self.ncols += 1;
        true
    }

    // 1922:18e3
    pub fn draw(&self, g: &mut Sre, last: bool, width: i16, colw: &[i16; 5]) {
        if display_len(g, &self.text) == 0 {
            return;
        }
        if self.key != 0 {
            let (o, c) = (g.menuchars.itemopen, g.menuchars.itemclose);
            kout!(g; byte_color(self.colors[0]), o, byte_color(self.colors[1]), self.key, byte_color(self.colors[0]), c);
        } else {
            kout!(g; "   ");
        }
        kout!(g; byte_color(self.colors[2]), b' ');
        if self.ncols > 0 || !last {
            kout!(g; setw(width + 1));
        }
        kout!(g; &self.text);
        kout!(g; setw(0));
        for k in 0..self.ncols.max(0) as usize {
            let cw = colw[k];
            kout!(g; byte_color(self.colors[3 + k]));
            if self.coltype[k] == b's' {
                kout!(g; setw(cw.wrapping_neg()));
                match &self.colstr[k] {
                    Some(s) => put_untranslated(g, KOUT, s),
                    None => kout!(g; " "),
                }
                kout!(g; setw(0));
            } else {
                kout!(g; setw(cw));
                if self.colnum[k] < 0 {
                    kout!(g; "-");
                } else {
                    kout!(g; self.colnum[k]);
                }
                kout!(g; setw(0));
            }
        }
        if last {
            kout!(g; b'\n');
        }
    }

    // 1922:1bea
    pub fn total_width(&self, g: &mut Sre, width: i16, colw: &[i16; 5]) -> i16 {
        if self.ncols == 0 {
            return display_len(g, &self.text) + 4;
        }
        let mut t = width + 5;
        for w in colw {
            t += *w;
        }
        t
    }

    // 1922:1c42
    pub fn col_width(&self, col: i16) -> i16 {
        if col > self.ncols {
            return 0;
        }
        let k = (col - 1) as usize;
        if self.coltype[k] == b's' {
            let l = self.colstr[k].as_ref().map_or(0, |s| s.len()) as i16;
            if l > 2 {
                return l + 2;
            }
            return l;
        }
        let mut v = self.colnum[k];
        let mut n: i16 = 0;
        while v > 0 {
            n += 1;
            v /= 10;
            if v > 0 {
                n += 1;
            }
            v /= 10;
            if v > 0 {
                n += 2;
            }
            v /= 10;
        }
        n + 2
    }
}
