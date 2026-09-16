#[derive(Debug)]
pub(crate) enum ReadError {
    UnexpectedEof,
}

pub(crate) struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Reader<'a> {
        Reader { data, position: 0 }
    }

    pub(crate) fn position(&self) -> usize {
        self.position
    }

    pub(crate) fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], ReadError> {
        let start = self.position;

        let end = match start.checked_add(length) {
            Some(value) => value,
            None => return Err(ReadError::UnexpectedEof),
        };

        if end > self.data.len() {
            return Err(ReadError::UnexpectedEof);
        }

        self.position = end;
        let slice: &'a [u8] = &self.data[start..end];

        Ok(slice)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, ReadError> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16, ReadError> {
        let bytes = self.read_bytes(2)?;

        let bytes: [u8; 2] = bytes
            .try_into()
            .expect("read_bytes returned an unexpected length");

        Ok(u16::from_be_bytes(bytes))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, ReadError> {
        let bytes = self.read_bytes(4)?;

        let bytes: [u8; 4] = bytes
            .try_into()
            .expect("read_bytes returned an unexpected length");

        Ok(u32::from_be_bytes(bytes))
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64, ReadError> {
        let bytes = self.read_bytes(8)?;

        let bytes: [u8; 8] = bytes
            .try_into()
            .expect("read_bytes returned an unexpected length");

        Ok(u64::from_be_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_bytes() {
        let data = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut reader = Reader::new(&data);

        let result = reader.read_bytes(2).unwrap();

        assert_eq!(result, &data[0..2]);
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn reads_consecutively() {
        let data = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut reader = Reader::new(&data);

        let first = reader.read_bytes(2).unwrap();

        assert_eq!(first, &data[0..2]);
        assert_eq!(reader.position(), 2);

        let second = reader.read_bytes(1).unwrap();

        assert_eq!(second, &data[2..3]);
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn rejects_read_past_end() {
        let data = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut reader = Reader::new(&data);

        let first = reader.read_bytes(3).unwrap();

        assert_eq!(first, &data[0..3]);
        assert_eq!(reader.position(), 3);

        let result = reader.read_bytes(2);

        assert!(result.is_err());
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn reads_zero_bytes() {
        let data = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut reader = Reader::new(&data);

        let result = reader.read_bytes(0).unwrap();

        assert_eq!(result, &[]);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reads_u8() {
        let data = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut reader = Reader::new(&data);

        let result = reader.read_u8().unwrap();
        let expected = 0xAA;

        assert_eq!(result, expected);
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn rejects_u8_at_end() {
        let data = [];
        let mut reader = Reader::new(&data);

        let result = reader.read_u8();

        assert!(result.is_err());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reads_u16_big_endian() {
        let data = [0x12, 0x34, 0xAA, 0xBB];
        let mut reader = Reader::new(&data);

        let result = reader.read_u16().unwrap();

        assert_eq!(result, 0x1234);
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn rejects_u16_at_end() {
        let data = [0x12];
        let mut reader = Reader::new(&data);

        let result = reader.read_u16();

        assert!(result.is_err());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reads_u32_big_endian() {
        let data = [0x12, 0x34, 0x56, 0x78, 0xAA];
        let mut reader = Reader::new(&data);

        let result = reader.read_u32().unwrap();

        assert_eq!(result, 0x12345678);
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn rejects_u32_at_end() {
        let data = [0x12, 0x34, 0x56];
        let mut reader = Reader::new(&data);

        let result = reader.read_u32();

        assert!(result.is_err());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reads_u64_big_endian() {
        let data = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xAA];
        let mut reader = Reader::new(&data);

        let result = reader.read_u64().unwrap();

        assert_eq!(result, 0x0123456789ABCDEF);
        assert_eq!(reader.position(), 8);
    }

    #[test]
    fn rejects_u64_at_end() {
        let data = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD];
        let mut reader = Reader::new(&data);

        let result = reader.read_u64();

        assert!(result.is_err());
        assert_eq!(reader.position(), 0);
    }
}
