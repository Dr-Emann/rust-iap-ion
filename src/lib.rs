use std::io;
use std::cmp;

mod types;

pub struct Writer<W> {
    inner: W
}

impl<W: io::Write> Writer<W> {
    pub fn new(inner: W) -> Writer<W> {
        Writer{ inner: inner }
    }

    pub fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_bool_opt(Some(value))
    }

    pub fn write_bool_opt(&mut self, value: Option<bool>) -> io::Result<()> {
        let raw_value = match value {
            None => 0,
            Some(true) => 1,
            Some(false) => 2,
        };
        self.write_tiny(types::BOOLEAN, raw_value)
    }

    pub fn write_int_pos(&mut self, value: u64) -> io::Result<()> {
        self.write_int_pos_opt(Some(value))
    }

    pub fn write_int_pos_opt(&mut self, value: Option<u64>) -> io::Result<()> {
        let mut value = match value {
            Some(value) => value,
            None => return self.write_short(types::INT_POS, &[])
        };
        let mut buffer = [0u8; 8];
        // round down, because we only want whole bytes
        let leading_bytes = (value.leading_zeros() / 8) as usize;
        let full_bytes = 8 - leading_bytes;
        for i in 0..full_bytes {
            let shift = 8 * (full_bytes - 1 - i);
            let byte = ((value & (0xFF << shift)) >> shift) as u8;
            buffer[i] = byte;
        }
        let len = cmp::max(1, full_bytes);
        self.write_short(types::INT_POS, &buffer[..len])
    }

    #[inline]
    fn write_tiny(&mut self, kind: u8, value: u8) -> io::Result<()> {
        debug_assert!(kind & 0xF0 == 0, "Kind must be < 16");
        debug_assert!(value & 0xF0 == 0, "value must be < 16");
        let header = kind << 4 | value;
        self.inner.write_all(&[header])
    }

    #[inline]
    fn write_short(&mut self, kind: u8, bytes: &[u8]) -> io::Result<()> {
        debug_assert!(bytes.len() <= 0xF);
        let header = kind << 4 | (bytes.len() as u8);
        self.inner.write_all(&[header])?;
        self.inner.write_all(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::Writer;
    #[test]
    fn write_boolean() {
        let mut data = Vec::new();
        {
            let mut writer = Writer::new(&mut data);
            writer.write_bool(true);
            writer.write_bool(false);
            writer.write_bool_opt(None);
        }
        assert_eq!(&data[..], &[0x11, 0x12, 0x10])
    }

    #[test]
    fn write_positive_int() {
        let mut data = Vec::new();
        {
            let mut writer = Writer::new(&mut data);
            writer.write_int_pos(0);
            writer.write_int_pos(u64::max_value());
            writer.write_int_pos(0x1234);
            writer.write_int_pos_opt(None);
        }
        assert_eq!(&data[..], &[
            0x21, 0x00,
            0x28, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x22, 0x12, 0x34,
            0x20
        ]);
    }
}
