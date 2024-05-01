use std::io::{Error, Write};


///Recibe un string y el stream al que escribir ese stream
/// 
/// Calcula su largo y luego escribe el largo y el string en el stream 
pub fn write_string(stream: &mut dyn Write, string: &str) -> Result<(), Error> {
    let length = string.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write(&length_bytes)?;
    stream.write(string.as_bytes())?;
    Ok(())
}

pub fn write_u16(stream: &mut dyn Write, value: &u16) -> Result<(), Error> {
    let value_bytes = value.to_be_bytes();
    stream.write(&value_bytes)?;
    Ok(())
}

pub fn write_u32(stream: &mut dyn Write, value: &u32) -> Result<(), Error> {
    let value_bytes = value.to_be_bytes();
    stream.write(&value_bytes)?;
    Ok(())
}
