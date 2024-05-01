use std::io::{Error, Read};


pub fn read_string(stream: &mut dyn Read)-> Result<String, Error>{
    let string_length = read_u16(stream)?;
    let mut string_buf = vec![0; string_length as usize];
    stream.read_exact(&mut string_buf)?;

    let protocol_name =
        std::str::from_utf8(&string_buf).expect("Error al leer protocol_name");
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
