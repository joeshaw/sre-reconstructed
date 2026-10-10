// "5 soldiers, 2 fighters, and 1 general" (segment 15dd).

use crate::prelude::*;

#[derive(Clone)]
pub struct ItemList {
    pub skipzero: bool,          // +0
    pub items: Vec<(Str, i32)>,  // +4 (at most 0x20)
    pub text: Color,             // +8
    pub number: Color,           // +c
}

impl ItemList {
    // 15dd:000c
    pub fn new(text: Color, number: Color) -> ItemList {
        ItemList { skipzero: false, items: Vec::new(), text, number }
    }

    // 15dd:00d3
    pub fn skip_zero(&mut self) -> &mut Self {
        self.skipzero = true;
        self
    }

    pub fn count(&self) -> i16 {
        self.items.len() as i16
    }

    // 15dd:00e8: plurals add an 's'.
    pub fn add(&mut self, n: i32, name: &[u8]) -> &mut Self {
        self.add_plural(n, name, b"s")
    }

    // 15dd:01e8: plurals add the given suffix.
    pub fn add_plural(&mut self, n: i32, name: &[u8], plural: &[u8]) -> &mut Self {
        if self.items.len() >= 0x20 {
            return self;
        }
        if self.skipzero && n == 0 {
            return self;
        }
        let mut s = name.to_vec();
        if n != 1 {
            s.extend_from_slice(plural);
        }
        self.items.push((s, n));
        self
    }
}

// 15dd:0368
pub fn put_item_list(g: &mut Sre, o: Sid, l: &ItemList) {
    let count = l.count();
    if count == 0 {
        out!(g, o; l.text, "nothing");
        return;
    }
    for i in 0..count {
        if count - 1 == i && count > 1 {
            out!(g, o; l.text, "and ");
        }
        let (name, n) = l.items[i as usize].clone();
        out!(g, o; l.number, n, b' ', l.text, name);
        if count - 1 > i {
            if count > 2 {
                out!(g, o; b',');
            }
            out!(g, o; b' ');
        }
    }
}
