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
        let value = match value {
            Some(value) => value,
            None => return self.write_short(types::INT_POS, &[])
        };
        let mut buffer = [0u8; 8];
        let len = build_int_buf(value, &mut buffer);
        self.write_short(types::INT_POS, &buffer[..len])
    }

    pub fn write_int_neg(&mut self, value: u64) -> io::Result<()> {
        self.write_int_neg_opt(Some(value))
    }

    pub fn write_int_neg_opt(&mut self, value: Option<u64>) -> io::Result<()> {
        let value = match value {
            Some(value) => value,
            None => return self.write_short(types::INT_NEG, &[])
        };
        let value = value.checked_sub(1).expect("cannot have negative 0");
        let mut buffer = [0u8; 8];
        let len = build_int_buf(value, &mut buffer);
        self.write_short(types::INT_NEG, &buffer[..len])
    }

    pub fn write_int(&mut self, value: i64) -> io::Result<()> {
        self.write_int_opt(Some(value))
    }

    pub fn write_int_opt(&mut self, value: Option<i64>) -> io::Result<()> {
        let value = match value {
            Some(value) => value,
            None => return self.write_short(types::INT_POS, &[]),
        };
        if value >= 0 {
            self.write_int_pos(value as u64)
        } else {
            self.write_int_neg((-value) as u64)
        }
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

fn build_int_buf(value: u64, buf: &mut [u8]) -> usize {
    // round down, because we only want whole bytes
    let leading_bytes = (value.leading_zeros() / 8) as usize;
    let full_bytes = 8 - leading_bytes;
    for i in 0..full_bytes {
        let shift = 8 * (full_bytes - 1 - i);
        let byte = ((value & (0xFF << shift)) >> shift) as u8;
        buf[i] = byte;
    }
    cmp::max(1, full_bytes)
}


#[cfg(test)]
mod tests {
    use super::Writer;
    #[test]
    fn write_boolean() {
        let mut data = Vec::new();
        {
            let mut writer = Writer::new(&mut data);
            writer.write_bool(true).unwrap();
            writer.write_bool(false).unwrap();
            writer.write_bool_opt(None).unwrap();
        }
        assert_eq!(&data[..], &[0x11, 0x12, 0x10])
    }

    #[test]
    fn write_positive_int() {
        let mut data = Vec::new();
        {
            let mut writer = Writer::new(&mut data);
            writer.write_int_pos(0).unwrap();
            writer.write_int_pos(u64::max_value()).unwrap();
            writer.write_int_pos(0x1234).unwrap();
            writer.write_int_pos_opt(None).unwrap();
        }
        assert_eq!(&data[..], &[
            0x21, 0x00,
            0x28, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x22, 0x12, 0x34,
            0x20
        ]);
    }

    #[test]
    fn write_negitive_int() {
        let mut data = Vec::new();
        {
            let mut writer = Writer::new(&mut data);
            writer.write_int_neg(1).unwrap();
            writer.write_int_neg(0x1234).unwrap();
            writer.write_int_neg_opt(None).unwrap();
        }
        assert_eq!(&data[..], &[
            0x31, 0x00,
            0x32, 0x12, 0x33,
            0x30
        ]);
    }

    #[test]
    #[should_panic]
    fn write_negative_zero() {
        let mut data = Vec::new();
        let mut writer = Writer::new(&mut data);
        writer.write_int_neg(0).unwrap();
    }

    #[test]
    fn write_ints() {
        let mut data = Vec::new();
        {
            let mut writer = Writer::new(&mut data);
            writer.write_int(0x01020304).unwrap();
            writer.write_int(-1).unwrap();
            writer.write_int(-257).unwrap();
            writer.write_int_opt(None).unwrap();
        }
        assert_eq!(&data[..], &[
            0x24, 0x01, 0x02, 0x03, 0x04,
            0x31, 0x00,
            0x32, 0x01, 0x00,
            0x20
        ]);
    }
}
