// Borland C++ 3.1 character classification.  The game's arithmetic uses
// explicit widths (i16 for the original's int, i32 for its long) and
// wraps, as the original's did.

// Borland's _ctype[] classification bits.
pub const CT_SP: u8 = 1;
pub const CT_DIG: u8 = 2;
pub const CT_UPP: u8 = 4;
pub const CT_LOW: u8 = 8;
#[allow(dead_code)]
pub const CT_HEX: u8 = 16;
pub const CT_CTL: u8 = 32;
#[allow(dead_code)]
pub const CT_PUN: u8 = 64;

// _ctype (DS:c2ab).  Index -128..-1 are the bytes that precede the table
// in DGROUP at run time; a few original routines index with a
// sign-extended char and so see these for characters >= 0x80.
static CTYPE_TAB: [u8; 128] = [
    32, 32, 32, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 32, 32, //
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, //
    1, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, //
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 64, 64, 64, 64, 64, 64, //
    64, 20, 20, 20, 20, 20, 20, 4, 4, 4, 4, 4, 4, 4, 4, 4, //
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 64, 64, 64, 64, 64, //
    64, 24, 24, 24, 24, 24, 24, 8, 8, 8, 8, 8, 8, 8, 8, 8, //
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 64, 64, 64, 64, 32, //
];
static CTYPE_BEFORE: [u8; 128] = [
    78, 79, 0, 0, 82, 69, 83, 79, 85, 82, 67, 69, 46, 68, 65, 84, //
    0, 82, 69, 83, 79, 85, 82, 67, 69, 46, 68, 69, 70, 0, 35, 69, //
    79, 70, 0, 116, 114, 117, 101, 0, 116, 0, 111, 110, 0, 121, 101, 115, //
    0, 200, 212, 85, 45, 216, 212, 85, 45, 184, 212, 85, 45, 0, 35, 60, //
    101, 114, 114, 111, 114, 62, 0, 32, 41, 0, 35, 60, 103, 97, 114, 98, //
    97, 103, 101, 62, 0, 47, 0, 12, 42, 82, 0, 12, 42, 154, 12, 12, //
    42, 69, 0, 12, 42, 58, 0, 12, 42, 210, 12, 12, 42, 47, 0, 12, //
    42, 58, 0, 12, 42, 6, 0, 12, 42, 0, 0, 0, 0, 0, 0, 0, //
];

// _ctype lookup with an unsigned index (the usual case).
pub fn ctype_of(c: u8) -> u8 {
    if c < 0x80 {
        CTYPE_TAB[c as usize]
    } else {
        0
    }
}

// _ctype lookup with a sign-extended char index, as some original code
// does; for c >= 0x80 this reads the bytes in front of the table.
pub fn ctype_signed(c: u8) -> u8 {
    if c < 0x80 {
        CTYPE_TAB[c as usize]
    } else {
        CTYPE_BEFORE[(c - 0x80) as usize]
    }
}

pub fn b_isupper(c: u8) -> bool {
    ctype_of(c) & CT_UPP != 0
}
pub fn b_islower(c: u8) -> bool {
    ctype_of(c) & CT_LOW != 0
}

// Borland toupper/tolower (1000:0fa1, 1000:0f75): ASCII only.  The
// original passes a sign-extended char; the result is the byte.
pub fn b_toupper(c: u8) -> u8 {
    if b_islower(c) {
        c - 32
    } else {
        c
    }
}
pub fn b_tolower(c: u8) -> u8 {
    if b_isupper(c) {
        c + 32
    } else {
        c
    }
}
