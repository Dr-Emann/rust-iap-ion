pub const BYTES: u8 = 0x0;
pub const BOOLEAN: u8 = 0x1;
pub const INT_POS: u8 = 0x2;
pub const INT_NEG: u8 = 0x3;
pub const FLOAT: u8 = 0x4;
pub const UTF8: u8 = 0x5;
pub const UTF8_SHORT: u8 = 0x6;
pub const UTC_DATE_TIME: u8 = 0x7;

pub const ARRAY: u8 = 0xA;
pub const TABLE: u8 = 0xB;
pub const OBJECT: u8 = 0xC;
pub const KEY: u8 = 0xD;
pub const KEY_SHORT: u8 = 0xE;
pub const EXTENDED: u8 = 0xF;

pub fn get_type(byte: u8) -> u8 {
    byte >> 4
}
