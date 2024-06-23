use std::io::{Error, Read};

pub fn read_string(stream: &mut dyn Read) -> Result<String, Error> {
    let string_length = read_u16(stream)?;
    let mut string_buf = vec![0; string_length as usize];
    stream.read_exact(&mut string_buf)?;

    let protocol_name = std::str::from_utf8(&string_buf).expect("Error al leer protocol_name");
    Ok(protocol_name.to_string())
}

pub fn read_u8(stream: &mut dyn Read) -> Result<u8, Error> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

pub fn read_u16(stream: &mut dyn Read) -> Result<u16, Error> {
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn read_u32(stream: &mut dyn Read) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

pub fn read_string_pairs(stream: &mut dyn Read) -> Result<Vec<(String, String)>, Error> {
    let length = read_u16(stream)?;
    let mut pairs = Vec::with_capacity(length as usize);
    for _ in 0..length {
        let key = read_string(stream)?;
        let value = read_string(stream)?;
        pairs.push((key, value));
    }
    Ok(pairs)
}

pub fn read_bin_vec(reader: &mut dyn Read) -> Result<Vec<u8>, Error> {
    let length = read_u16(reader)?;
    let mut buffer = vec![0; length as usize];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

pub fn read_tuple_vec(reader: &mut dyn Read) -> Result<Vec<(String, String)>, Error> {
    let length = read_u16(reader)?;
    let mut vec = Vec::with_capacity(length as usize);
    for _ in 0..length {
        let key = read_string(reader)?;
        let value = read_string(reader)?;
        vec.push((key, value));
    }
    Ok(vec)
}
pub fn read_bool(stream: &mut dyn Read) -> Result<bool, Error> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    Ok(buf[0] != 0)
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_string() {
        let mut cursor = Cursor::new(vec![0, 4, 116, 101, 115, 116]);
        assert_eq!(read_string(&mut cursor).unwrap(), "test".to_string());
    }

    #[test]
    fn test_read_u8() {
        let mut cursor = Cursor::new(vec![1]);
        assert_eq!(read_u8(&mut cursor).unwrap(), 1);
    }

    #[test]
    fn test_read_u16() {
        let mut cursor = Cursor::new(vec![123, 128, 129]);
        assert_eq!(read_u16(&mut cursor).unwrap(), 31616);
    }

    #[test]
    fn test_read_u32() {
        let mut cursor = Cursor::new(vec![0, 1, 0, 0, 0, 2]);
        assert_eq!(read_u32(&mut cursor).unwrap(), 65536);
    }

    #[test]
    fn test_read_bin_vec() {
        let mut cursor = Cursor::new(vec![0, 1, 1]);
        assert_eq!(read_bin_vec(&mut cursor).unwrap(), vec![1]);
    }

    #[test]
    fn test_read_tuple_vec() {
        let mut cursor = Cursor::new(vec![
            0, 1, 0, 4, 116, 101, 115, 116, 0, 1, 0, 4, 116, 101, 115, 116, 0, 1, 0, 4, 116, 101,
            115, 116,
        ]);
        assert_eq!(
            read_tuple_vec(&mut cursor).unwrap(),
            vec![("test".to_string(), "\0".to_string())]
        );
    }

    #[test]
    fn test_read_bool() {
        let mut cursor = Cursor::new(vec![1]);
        assert!(read_bool(&mut cursor).unwrap());
    }
}
