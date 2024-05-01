use std::io::{Error, Write};

enum Type {
    String,
    U16,
    U32,
    Vec,
    Bool,
    StringTuple,
}

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

pub fn write_tuple_vec(stream: &mut dyn Write, vec: &Vec<(String, String)>) -> Result<(), Error> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write(&length_bytes)?;
    for item in vec {
        write_string_tuple(stream, item)?;
    }
    Ok(())
}

// pub fn write_vec<T>(stream: &mut dyn Write, vec: &Vec<T>, data_type: Type) -> Result<(), Error> {
//     match data_type {
//         Type::String => {
//             for item in vec {
//                 write_string(stream, &item.to_string())?;
//             }
//         },

//         Type::U16 => {},
//         Type::U32 => todo!(),
//         Type::Vec => todo!(),
//         Type::Bool => todo!(),
//         Type::StringTuple => todo!(),
//     }

//     Ok(())
// }

pub fn write_bin_vec(stream: &mut dyn Write, vec: &Vec<u8>) -> Result<(), Error> {
    let length = vec.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write(&length_bytes)?;
    for byte in vec {
        stream.write(&[*byte])?;
    }
    Ok(())
}

pub fn write_string_tuple(stream: &mut dyn Write, value: &(String, String)) -> Result<(), Error> {
    write_string(stream, &value.0)?;
    write_string(stream, &value.1)?;
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

pub fn write_bool(stream: &mut dyn Write, value: &bool) -> Result<(), Error> {
    let value_bytes = if *value { 1u8 } else { 0u8 };
    stream.write(&[value_bytes])?;
    Ok(())
}