use std::io::{Error, ErrorKind, Read};

pub fn read_string(stream: &mut dyn Read) -> Result<String, Error> {
    let string_length = match read_u16(stream) {
        Ok(s) => s,
        Err(e) => match e.kind() {
            ErrorKind::WouldBlock => {
                println!("la lectura falló");
                return Err(Error::from(ErrorKind::WouldBlock));
            }
            _ => return Err(e),
        },
    };
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
    let mut pairs = Vec::new();
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
