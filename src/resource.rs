// RESOURCE.DAT configuration system (segments 2932 and 2a0c).
//
// Each line of RESOURCE.<node>, RESOURCE.DAT and RESOURCE.DEF is read
// with a small Lisp-like reader: the first token must be a symbol (the
// resource name) and the rest of the line is its value list.  A Resource
// object looks itself up lazily: first in the command line (/Name,
// /Name=1, /NoName), then in the loaded lines, where the first line
// loaded wins.

use crate::brtl::{b_atol, b_ltoa, b_stricmp, b_strnicmp};
use crate::files::{read_line_file, FileIn};
use crate::sstring::{at, str_eq_i, Str};
use crate::types::*;
use std::rc::Rc;

pub const AT_ERROR: i16 = 0;
pub const AT_SYMBOL: i16 = 1;
pub const AT_QSYMBOL: i16 = 2;
pub const AT_STRING: i16 = 3;
pub const AT_NUMBER: i16 = 4;
pub const AT_BOOL: i16 = 5;

// The shared constants nil, #t and #f.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Shared {
    No,
    Nil,
    True,
    False,
}

// An atom (string/number) or a cons cell (list).
pub enum Node {
    Atom { s: Str, num: i32, typ: i16, shared: Shared },
    Cons { car: Option<Rc<Node>>, cdr: Option<Rc<Node>> },
}

pub type NodeP = Option<Rc<Node>>;

fn atom(s: Str, num: i32, typ: i16) -> Rc<Node> {
    Rc::new(Node::Atom { s, num, typ, shared: Shared::No })
}

pub fn nil() -> Rc<Node> {
    Rc::new(Node::Atom { s: Str::new(), num: 0, typ: AT_ERROR, shared: Shared::Nil })
}
fn tru() -> Rc<Node> {
    Rc::new(Node::Atom { s: Str::new(), num: 1, typ: AT_BOOL, shared: Shared::True })
}
fn fals() -> Rc<Node> {
    Rc::new(Node::Atom { s: Str::new(), num: 0, typ: AT_BOOL, shared: Shared::False })
}

impl Node {
    pub fn is_cons(&self) -> bool {
        matches!(self, Node::Cons { .. })
    }
    pub fn is_nil(&self) -> bool {
        matches!(self, Node::Atom { shared: Shared::Nil, .. })
    }
    pub fn str(&self) -> &[u8] {
        match self {
            Node::Atom { s, .. } => s,
            _ => b"",
        }
    }
    pub fn num(&self) -> i32 {
        match self {
            Node::Atom { num, .. } => *num,
            _ => 0,
        }
    }
    pub fn typ(&self) -> i16 {
        match self {
            Node::Atom { typ, .. } => *typ,
            _ => AT_ERROR,
        }
    }
}

// 2a0c:005f (nil if not a cons)
pub fn node_car(list: &NodeP) -> NodeP {
    match list.as_deref() {
        Some(Node::Cons { car, .. }) => car.clone(),
        _ => Some(nil()),
    }
}

// 2a0c:0082
pub fn node_cdr(list: &NodeP) -> NodeP {
    match list.as_deref() {
        Some(Node::Cons { cdr, .. }) => cdr.clone(),
        _ => None,
    }
}

// 2a0c:0148
pub fn node_nth(list: &NodeP, mut n: i16) -> NodeP {
    let mut p = list.clone();
    while n > 0 && p.is_some() {
        p = cdr_of(&p);
        n -= 1;
    }
    match p.as_deref() {
        None => Some(nil()),
        Some(Node::Cons { car, .. }) => car.clone(),
        // An atom where a cell is expected: its "car" field.
        Some(_) => None,
    }
}

fn cdr_of(p: &NodeP) -> NodeP {
    match p.as_deref() {
        Some(Node::Cons { cdr, .. }) => cdr.clone(),
        _ => None,
    }
}

// 2a0c:0116
pub fn node_length(list: &NodeP) -> i16 {
    let mut n = 0;
    let mut p = list.clone();
    while p.is_some() {
        n += 1;
        p = cdr_of(&p);
    }
    n
}

// The atom n, if it is one.
pub fn as_atom(n: &NodeP) -> Option<&Node> {
    match n.as_deref() {
        Some(a) if !a.is_cons() => Some(a),
        _ => None,
    }
}

// 2a0c:019a: read a symbol.
fn read_symbol(s: &[u8], pos: &mut i16) -> (Str, i16) {
    while at(s, *pos) != 0 && (ctype_signed(at(s, *pos)) & CT_SP) != 0 {
        *pos += 1;
    }
    let mut a = Str::new();
    loop {
        let c = at(s, *pos);
        if c == 0 {
            break;
        }
        if !((c as i8) > 0x20) {
            break;
        }
        if !(c < 0xff) {
            break;
        }
        if matches!(c, b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'\'' | b'"') {
            break;
        }
        a.push(c);
        *pos += 1;
    }
    (a, AT_SYMBOL)
}

// 2a0c:0828: read list items until a closing bracket or the end.
fn read_list(s: &[u8], pos: &mut i16) -> NodeP {
    let len = s.len() as i16;
    let mut items: Vec<NodeP> = Vec::new();
    while *pos < len {
        let item = read_item(s, pos);
        if item.as_deref().map_or(false, |n| n.is_nil()) {
            break;
        }
        items.push(item);
    }
    build_list(items)
}

fn build_list(items: Vec<NodeP>) -> NodeP {
    let mut head: NodeP = None;
    for item in items.into_iter().rev() {
        head = Some(Rc::new(Node::Cons { car: item, cdr: head }));
    }
    head
}

// 2a0c:031f
fn read_item(s: &[u8], pos: &mut i16) -> NodeP {
    while at(s, *pos) != 0 && (ctype_signed(at(s, *pos)) & CT_SP) != 0 {
        *pos += 1;
    }
    let mut c = at(s, *pos);
    if c == 0 {
        return None;
    }
    if c == b'"' {
        let mut a = Str::new();
        loop {
            *pos += 1;
            c = at(s, *pos);
            if c == 0 || c == b'"' {
                break;
            }
            if c == b'~' {
                *pos += 1;
                c = at(s, *pos);
                c = match c {
                    b'n' => b'\n',
                    b'b' => 8,
                    b't' => 9,
                    b'e' => 0x1b,
                    b'a' => 7,
                    b'v' => 0x0b,
                    b'r' => b'\r',
                    c => c,
                };
            }
            a.push(c);
        }
        if at(s, *pos) != b'"' {
            return Some(nil());
        }
        *pos += 1;
        return Some(atom(a, 0, AT_STRING));
    }
    if c == b'(' || c == b'[' || c == b'{' {
        *pos += 1;
        return read_list(s, pos);
    }
    if c == b')' || c == b']' || c == b'}' {
        *pos += 1;
        return Some(nil());
    }
    if c == b'#' {
        *pos += 1;
        c = at(s, *pos);
        if c == b'f' || c == b'F' {
            *pos += 1;
            return Some(fals());
        }
        if c == b't' || c == b'T' {
            *pos += 1;
            return Some(tru());
        }
        return Some(nil());
    }
    if c == b'\'' {
        *pos += 1;
        let (a, _) = read_symbol(s, pos);
        return Some(atom(a, 0, AT_QSYMBOL));
    }
    let (a, _) = read_symbol(s, pos);
    let f = at(&a, 0);
    if (ctype_signed(f) & CT_DIG) != 0 || ((f == b'+' || f == b'-') && at(&a, 1) != 0) {
        for i in 1..a.len() {
            if ctype_signed(a[i]) & CT_DIG == 0 {
                return Some(atom(a, 0, AT_SYMBOL));
            }
        }
        let num = b_atol(&a);
        return Some(atom(a, num, AT_NUMBER));
    }
    Some(atom(a, 0, AT_SYMBOL))
}

// 2a0c:0b1b: parse a whole line into a list.
pub fn parse_list(s: &[u8]) -> NodeP {
    let mut pos: i16 = 0;
    let len = s.len() as i16;
    let mut items: Vec<NodeP> = Vec::new();
    while pos < len {
        let item = read_item(s, &mut pos);
        if item.is_none() {
            break;
        }
        let isnil = item.as_deref().map_or(false, |n| n.is_nil());
        items.push(item);
        if isnil {
            break;
        }
    }
    build_list(items)
}

// ---------------------------------------------------------------------
// The loaded resource lines.

#[derive(Default)]
pub struct ResDb {
    list: Vec<(Str, NodeP)>, // the first line loaded first
    loaded: bool,
    node: Str,
    pub argv: Vec<Str>,
}

impl ResDb {
    // 2932:0912
    pub fn load_file(&mut self, path: &[u8]) {
        let mut f = FileIn::open(path, false);
        if f.fail() {
            return;
        }
        loop {
            let line = read_line_file(&mut f);
            if f.fail() {
                break;
            }
            if str_eq_i(b"#EOF", &line) {
                break;
            }
            if matches!(at(&line, 0), b';' | b'#' | b'%') {
                continue;
            }
            let tokens = parse_list(&line);
            let first = node_nth(&tokens, 0);
            let rest = node_cdr(&tokens);
            match first.as_deref() {
                Some(n) if !n.is_nil() && !n.is_cons() && n.typ() == AT_SYMBOL => {
                    // Entries go in front.
                    self.list.insert(0, (n.str().to_vec(), rest));
                }
                _ => continue,
            }
        }
    }

    // 2932:0cba: -nX selects RESOURCE.X
    fn parse_node_option(&mut self) {
        for a in self.argv.iter().skip(1) {
            let at = |i: usize| a.get(i).cloned().unwrap_or(0);
            if at(0) != b'/' && at(0) != b'-' {
                continue;
            }
            if (at(1) == b'n' || at(1) == b'N') && at(2) != 0 {
                let mut n = vec![at(2), at(3), if at(3) != 0 { at(4) } else { 0 }];
                while n.last() == Some(&0) {
                    n.pop();
                }
                if let Some(z) = n.iter().position(|&c| c == 0) {
                    n.truncate(z);
                }
                self.node = n;
            }
        }
    }

    pub fn load(&mut self) {
        if self.loaded {
            return;
        }
        self.parse_node_option();
        self.loaded = true;
        if !self.node.is_empty() {
            let mut f = b"RESOURCE.".to_vec();
            f.extend_from_slice(&self.node);
            self.load_file(&f);
        }
        self.load_file(b"RESOURCE.DAT");
        self.load_file(b"RESOURCE.DEF");
    }

    // 2932:0c2c: is a plain (non-switch) argument equal to s?
    pub fn has_command_arg(&self, s: &[u8]) -> bool {
        self.argv.iter().skip(1).any(|a| {
            let c = a.first().cloned().unwrap_or(0);
            c != b'/' && c != b'-' && b_stricmp(a, s) == 0
        })
    }
}

pub struct Resource {
    pub name: Str,
    pub value: NodeP,
    pub resolved: bool,
}

impl Resource {
    pub fn text(name: &[u8], def: Option<&[u8]>) -> Resource {
        let mut r = Resource { name: name.to_vec(), value: None, resolved: false };
        r.set_default(def);
        r
    }

    pub fn num(name: &[u8], def: i32) -> Resource {
        let buf = b_ltoa(def, 10);
        Resource::text(name, Some(&buf))
    }

    // 2932:06c0
    pub fn set_default(&mut self, s: Option<&[u8]>) {
        self.value = Some(match s {
            Some(s) => {
                let num = b_atol(s);
                atom(s.to_vec(), num, if num != 0 { AT_NUMBER } else { AT_STRING })
            }
            None => atom(Str::new(), 0, AT_STRING),
        });
    }

    // 2932:0467
    pub fn resolve(&mut self, db: &mut ResDb) {
        db.load();
        if self.resolved {
            return;
        }
        for a in db.argv.iter().skip(1) {
            let mut v: i16 = -1;
            let c = a.first().cloned().unwrap_or(0);
            if c != b'/' && c != b'-' {
                continue;
            }
            let mut buf: Str = a[1..].to_vec();
            buf.truncate(0xff);
            let (name, val): (Str, Str) = match buf.iter().position(|&c| c == b'=') {
                Some(p) => (buf[..p].to_vec(), buf[p + 1..].to_vec()),
                None => (buf.clone(), Str::new()),
            };
            let p0 = val.first().cloned().unwrap_or(0);
            if b_stricmp(&name, &self.name) == 0 {
                v = 1;
                if p0 == b'0' || b_toupper(p0) == b'F' || b_toupper(p0) == b'N' {
                    v = 0;
                }
            }
            if b_strnicmp(&name, b"NO", 2) == 0 && b_stricmp(name.get(2..).unwrap_or(b""), &self.name) == 0 {
                v = 0;
            }
            if v != -1 {
                let s = [b'0' + v as u8];
                self.set_default(Some(&s));
                self.resolved = true;
            }
        }
        if self.resolved {
            return;
        }
        for (name, value) in db.list.iter() {
            if b_stricmp(&self.name, name) == 0 {
                self.value = value.clone();
                self.resolved = true;
            }
        }
    }

    // 2932:0225
    pub fn long(&mut self, db: &mut ResDb) -> i32 {
        self.resolve(db);
        match self.value.as_deref() {
            None => 0,
            Some(Node::Atom { num, .. }) => *num,
            Some(_) => match node_car(&self.value).as_deref() {
                Some(c) if !c.is_cons() => c.num(),
                _ => 0,
            },
        }
    }

    // 2932:02e0
    pub fn list(&mut self, db: &mut ResDb) -> NodeP {
        self.resolve(db);
        match self.value.as_deref() {
            None => None,
            Some(n) if !n.is_cons() => self.value.clone(),
            Some(_) => {
                if node_cdr(&self.value).is_none() {
                    node_car(&self.value)
                } else {
                    self.value.clone()
                }
            }
        }
    }

    // 2932:038b
    pub fn str(&mut self, db: &mut ResDb) -> Str {
        self.resolve(db);
        match self.value.as_deref() {
            None => Str::new(),
            Some(Node::Atom { s, .. }) => s.clone(),
            Some(_) => match node_car(&self.value).as_deref() {
                Some(c) if !c.is_cons() => c.str().to_vec(),
                _ => Str::new(),
            },
        }
    }

    // 2932:0af0
    pub fn bool(&mut self, db: &mut ResDb) -> bool {
        self.resolve(db);
        let v = match &self.value {
            None => return false,
            Some(v) => v.clone(),
        };
        let a = if v.is_cons() { node_car(&self.value) } else { Some(v) };
        let a = match a.as_deref() {
            Some(a) if !a.is_cons() => a,
            _ => return false,
        };
        if a.typ() == AT_NUMBER && a.num() != 0 {
            return true;
        }
        if a.typ() == AT_BOOL {
            return a.num() != 0;
        }
        let s = a.str();
        str_eq_i(b"true", s) || str_eq_i(b"t", s) || str_eq_i(b"on", s) || str_eq_i(b"yes", s)
    }
}
