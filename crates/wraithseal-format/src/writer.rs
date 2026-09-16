pub(crate) struct Writer {
    data: Vec<u8>,
}

impl Writer {
    pub(crate) fn position(&self) -> usize {
        self.data.len()
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub(crate) fn new() -> Self {
        Writer { data: Vec::new() }
    }

    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub(crate) fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    pub(crate) fn write_u16(&mut self, value: u16) {
        let bytes = value.to_be_bytes();
        self.write_bytes(&bytes);
    }

    pub(crate) fn write_u32(&mut self, value: u32) {
        let bytes = value.to_be_bytes();
        self.write_bytes(&bytes);
    }

    pub(crate) fn write_u64(&mut self, value: u64) {
        let bytes = value.to_be_bytes();
        self.write_bytes(&bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_bytes() {
        let mut writer = Writer::new();

        writer.write_bytes(&[0xAA, 0xBB]);

        assert_eq!(writer.as_slice(), &[0xAA, 0xBB]);
        assert_eq!(writer.position(), 2);
    }

    #[test]
    fn writes_consecutively() {
        let mut writer = Writer::new();

        writer.write_bytes(&[0xAA, 0xBB]);
        writer.write_bytes(&[0xCC, 0xDD]);

        assert_eq!(writer.as_slice(), &[0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(writer.position(), 4);
    }

    #[test]
    fn writes_u8() {
        let mut writer = Writer::new();

        writer.write_u8(0xAA);

        assert_eq!(writer.as_slice(), &[0xAA]);
        assert_eq!(writer.position(), 1);
    }

    #[test]
    fn writes_u16_big_endian() {
        let mut writer = Writer::new();

        writer.write_u16(0x1234);

        assert_eq!(writer.as_slice(), &[0x12, 0x34]);
        assert_eq!(writer.position(), 2);
    }

    #[test]
    fn writes_u32_big_endian() {
        let mut writer = Writer::new();

        writer.write_u32(0x12345678);

        assert_eq!(writer.as_slice(), &[0x12, 0x34, 0x56, 0x78]);
        assert_eq!(writer.position(), 4);
    }

    #[test]
    fn writes_u64_big_endian() {
        let mut writer = Writer::new();

        writer.write_u64(0x0123456789ABCDEF);

        assert_eq!(
            writer.as_slice(),
            &[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]
        );
        assert_eq!(writer.position(), 8);
    }
}
