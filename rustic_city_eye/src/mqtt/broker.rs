//! Abre un puerto TCP en el puerto asignado por argv.
//! Escribe las lineas recibidas a stdout y las manda mediante el socket.

mod client;

use std::env::args;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

static SERVER_ARGS: usize = 2;

fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(());
    }

    let address = "127.0.0.1:".to_owned() + &argv[1];
    server_run(&address).unwrap();
    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    let (mut client_stream, socket_addr) = listener.accept()?;
    println!("La socket addr del client: {:?}", socket_addr);
    handle_client(&mut client_stream)?;
    Ok(())
}

fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    let cloned_stream = stream.try_clone()?; // Clone the TcpStream
    let reader = BufReader::new(cloned_stream); // Use the cloned stream in BufReader
    let mut lines = reader.lines();
    while let Some(Ok(line)) = lines.next() {
        if line == "hola" {
            stream.write_all(b"chau\n")?;
        } else if line == "wasaa" {
            stream.write_all(b"wasaa\n")?;
        } else {
            stream.write_all(b"no entiendo\n")?;
        }
    }
    Ok(())
}
