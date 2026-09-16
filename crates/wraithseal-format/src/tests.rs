use crate::reader::Reader;
use crate::writer::Writer;

#[test]
fn preserves_bytes_after_write_and_read() {
    let original = [0xAA, 0xBB, 0xCC, 0xDD];

    let mut writer = Writer::new();
    writer.write_bytes(&original);

    let mut reader = Reader::new(writer.as_slice());
    let result = reader.read_bytes(original.len()).unwrap();

    assert_eq!(result, original);
    assert_eq!(reader.position(), writer.position());
}

#[test]
fn preserves_u8_after_write_and_read() {
    let original = 0xAB;

    let mut writer = Writer::new();
    writer.write_u8(original);

    let mut reader = Reader::new(writer.as_slice());
    let result = reader.read_u8().unwrap();

    assert_eq!(result, original);
    assert_eq!(reader.position(), writer.position());
}

#[test]
fn preserves_u16_after_write_and_read() {
    let original = 0x1234;

    let mut writer = Writer::new();
    writer.write_u16(original);

    let mut reader = Reader::new(writer.as_slice());
    let result = reader.read_u16().unwrap();

    assert_eq!(result, original);
    assert_eq!(reader.position(), writer.position());
}

#[test]
fn preserves_u32_after_write_and_read() {
    let original = 0x12345678;

    let mut writer = Writer::new();
    writer.write_u32(original);

    let mut reader = Reader::new(writer.as_slice());
    let result = reader.read_u32().unwrap();

    assert_eq!(result, original);
    assert_eq!(reader.position(), writer.position());
}

#[test]
fn preserves_u64_after_write_and_read() {
    let original = 0x0123456789ABCDEF;

    let mut writer = Writer::new();
    writer.write_u64(original);

    let mut reader = Reader::new(writer.as_slice());
    let result = reader.read_u64().unwrap();

    assert_eq!(result, original);
    assert_eq!(reader.position(), writer.position());
}

#[test]
fn preserves_mixed_values_after_write_and_read() {
    let original_u8 = 0xAA;
    let original_u16 = 0x1234;
    let original_u32 = 0x56789ABC;
    let original_u64 = 0x0123456789ABCDEF;

    let mut writer = Writer::new();

    writer.write_u8(original_u8);
    writer.write_u16(original_u16);
    writer.write_u32(original_u32);
    writer.write_u64(original_u64);

    let mut reader = Reader::new(writer.as_slice());

    assert_eq!(reader.read_u8().unwrap(), original_u8);
    assert_eq!(reader.read_u16().unwrap(), original_u16);
    assert_eq!(reader.read_u32().unwrap(), original_u32);
    assert_eq!(reader.read_u64().unwrap(), original_u64);

    assert_eq!(reader.position(), writer.position());
}
