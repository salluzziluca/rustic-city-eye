use std::io::Write;

use crate::protocol_error::ProtocolError;

///Recibe un string y el stream al que escribir ese stream
///
/// Calcula su largo y luego escribe el largo y el string en el stream
pub fn write_string(stream: &mut dyn Write, string: &str) -> Result<(), ProtocolError> {
    let length = string.len() as u16;
    let length_bytes = length.to_be_bytes();
    let _ = stream
        .write_all(&length_bytes)
        .map_err(|_e| ProtocolError::WriteError);
    let _ = stream
        .write_all(string.as_bytes())
        .map_err(|_e| ProtocolError::WriteError);
    Ok(())
}

pub fn write_tuple_vec(
    stream: &mut dyn Write,
    vec: &Vec<(String, String)>,
) -> Result<(), ProtocolError> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    let _ = stream
        .write_all(&length_bytes)
        .map_err(|_e| ProtocolError::WriteError);
    for item in vec {
        let _ = write_string_tuple(stream, item).map_err(|_e| ProtocolError::WriteError);
    }
    Ok(())
}

pub fn write_bin_vec(stream: &mut dyn Write, vec: &Vec<u8>) -> Result<(), ProtocolError> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    let _ = stream
        .write_all(&length_bytes)
        .map_err(|_e| ProtocolError::WriteError);
    for byte in vec {
        let _ = stream
            .write_all(&[*byte])
            .map_err(|_e| ProtocolError::WriteError);
    }
    Ok(())
}

pub fn write_string_tuple(
    stream: &mut dyn Write,
    value: &(String, String),
) -> Result<(), ProtocolError> {
    let _ = write_string(stream, &value.0).map_err(|_e| ProtocolError::WriteError);
    let _ = write_string(stream, &value.1).map_err(|_| ProtocolError::WriteError);
    Ok(())
}

pub fn write_u8(stream: &mut dyn Write, value: &u8) -> Result<(), ProtocolError> {
    let _ = stream
        .write_all(&[*value])
        .map_err(|_| ProtocolError::WriteError);
    Ok(())
}

pub fn write_u16(stream: &mut dyn Write, value: &u16) -> Result<(), ProtocolError> {
    let value_bytes = value.to_be_bytes();
    let _ = stream
        .write_all(&value_bytes)
        .map_err(|_| ProtocolError::WriteError);
    Ok(())
}

pub fn write_u32(stream: &mut dyn Write, value: &u32) -> Result<(), ProtocolError> {
    let value_bytes = value.to_be_bytes();
    let _ = stream
        .write_all(&value_bytes)
        .map_err(|_e| ProtocolError::WriteError);
    Ok(())
}

pub fn write_bool(stream: &mut dyn Write, value: &bool) -> Result<(), ProtocolError> {
    let value_bytes = if *value { 1u8 } else { 0u8 };
    let _ = stream
        .write_all(&[value_bytes])
        .map_err(|_e| ProtocolError::WriteError);
    Ok(())
}

pub fn write_string_pairs(
    stream: &mut dyn Write,
    vec: &Vec<(String, String)>,
) -> Result<(), ProtocolError> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    let _ = stream
        .write_all(&length_bytes)
        .map_err(|_e| ProtocolError::WriteError);
    for pair in vec {
        let _ = write_string(stream, &pair.0).map_err(|_| ProtocolError::WriteError);
        let _ = write_string(stream, &pair.1).map_err(|_e| ProtocolError::WriteError);
    }
    Ok(())
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn test_write_string() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        write_string(&mut cursor, "Hola").unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf.to_vec()).unwrap(), "Hola");
    }

    #[test]
    fn test_write_tuple_vec() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let vec = vec![("Hola".to_string(), "Chau".to_string())];
        write_tuple_vec(&mut cursor, &vec).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 1);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf.to_vec()).unwrap(), "Hola");
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf.to_vec()).unwrap(), "Chau");
    }

    #[test]
    fn test_write_bin_vec() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let vec = vec![1, 2, 3, 4];
        write_bin_vec(&mut cursor, &vec).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(buf.to_vec(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_write_string_tuple() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        write_string_tuple(&mut cursor, &("Hola".to_string(), "Chau".to_string())).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf.to_vec()).unwrap(), "Hola");
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf.to_vec()).unwrap(), "Chau");
    }

    #[test]
    fn test_write_u8() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        write_u8(&mut cursor, &1).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(buf[0], 1);
    }

    #[test]
    fn test_write_u16() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        write_u16(&mut cursor, &1).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 1);
    }

    #[test]
    fn test_write_u32() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        write_u32(&mut cursor, &1).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u32::from_be_bytes(buf), 1);
    }

    #[test]
    fn test_write_bool() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        write_bool(&mut cursor, &true).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(buf[0], 1);
    }

    #[test]
    fn test_write_string_pairs() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let vec = vec![("Hola".to_string(), "Chau".to_string())];
        write_string_pairs(&mut cursor, &vec).unwrap();
        cursor.set_position(0);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 1);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf.to_vec()).unwrap(), "Hola");
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(u16::from_be_bytes(buf), 4);
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf.to_vec()).unwrap(), "Chau");
    }
}
