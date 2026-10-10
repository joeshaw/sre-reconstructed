// What the game modules use.

pub use crate::brtl::*;
pub use crate::conio::*;
pub use crate::crypt::*;
pub use crate::devices::*;
pub use crate::files::*;
pub use crate::game::*;
pub use crate::layout::{get_i32, set_i32, Le};
pub use crate::menu::*;
pub use crate::news::*;
pub use crate::report::*;
pub use crate::resource::*;
pub use crate::screens::*;
pub use crate::sre::*;
pub use crate::sstring::*;
pub use crate::stream::*;
pub use crate::streamfile::*;
pub use crate::timeutil::*;
pub use crate::translate::*;
pub use crate::types::*;
pub use crate::{kout, out};

pub use crate::empire::*;
pub use crate::galaxy::*;
pub use crate::galaxymenu::*;
pub use crate::itemlist::*;
pub use crate::system::*;

// The colours the game keeps in variables.
pub fn cn(g: &Sre) -> Color {
    g.dev.c_normal
}
pub fn cin(g: &Sre) -> Color {
    g.dev.c_input
}
pub fn cnum(g: &Sre) -> Color {
    g.c.number
}
pub fn cbr(g: &Sre) -> Color {
    g.c.bracket
}
pub fn cbo(g: &Sre) -> Color {
    g.c.bold
}
pub fn cemp(g: &Sre) -> Color {
    g.c.empire
}

// An empire's name, as text to write (translated) or untranslated.
pub fn ename(g: &Sre, i: usize) -> Str {
    cs(&g.emp[i].name)
}
pub fn uname(g: &Sre, i: usize) -> Untr {
    Untr(cs(&g.emp[i].name))
}
