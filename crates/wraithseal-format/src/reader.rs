#[derive(Debug)]
enum ReadError {
    UnexpectedEof,
}

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Reader<'a> {
        Reader { data, position: 0 }
    }

    fn position(&self) -> usize {
        self.position
    }

    fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], ReadError> {
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
}
