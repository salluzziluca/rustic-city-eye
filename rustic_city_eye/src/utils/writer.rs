use std::io::{Error, Write};

///Recibe un string y el stream al que escribir ese stream
///
/// Calcula su largo y luego escribe el largo y el string en el stream
pub fn write_string(stream: &mut dyn Write, string: &str) -> Result<(), Error> {
    let length = string.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write_all(&length_bytes)?;
    stream.write_all(string.as_bytes())?;
    Ok(())
}

pub fn write_tuple_vec(stream: &mut dyn Write, vec: &Vec<(String, String)>) -> Result<(), Error> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write_all(&length_bytes)?;
    for item in vec {
        write_string_tuple(stream, item)?;
    }
    Ok(())
}

pub fn write_bin_vec(stream: &mut dyn Write, vec: &Vec<u8>) -> Result<(), Error> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write_all(&length_bytes)?;
    for byte in vec {
        stream.write_all(&[*byte])?;
    }
    Ok(())
}

pub fn write_string_tuple(stream: &mut dyn Write, value: &(String, String)) -> Result<(), Error> {
    write_string(stream, &value.0)?;
    write_string(stream, &value.1)?;
    Ok(())
}

pub fn write_u8(stream: &mut dyn Write, value: &u8) -> Result<(), Error> {
    stream.write_all(&[*value])?;
    Ok(())
}

pub fn write_u16(stream: &mut dyn Write, value: &u16) -> Result<(), Error> {
    let value_bytes = value.to_be_bytes();
    stream.write_all(&value_bytes)?;
    Ok(())
}

pub fn write_u32(stream: &mut dyn Write, value: &u32) -> Result<(), Error> {
    let value_bytes = value.to_be_bytes();
    stream.write_all(&value_bytes)?;
    Ok(())
}

pub fn write_bool(stream: &mut dyn Write, value: &bool) -> Result<(), Error> {
    let value_bytes = if *value { 1u8 } else { 0u8 };
    stream.write_all(&[value_bytes])?;
    Ok(())
}

pub fn write_string_pairs(
    stream: &mut dyn Write,
    vec: &Vec<(String, String)>,
) -> Result<(), Error> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write_all(&length_bytes)?;
    for pair in vec {
        write_string(stream, &pair.0)?;
        write_string(stream, &pair.1)?;
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
