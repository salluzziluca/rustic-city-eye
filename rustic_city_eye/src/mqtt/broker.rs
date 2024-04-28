use std::env::args;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use rustic_city_eye::mqtt::broker_message::BrokerMessage;
use rustic_city_eye::mqtt::client_message::ClientMessage;

mod client;

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

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                std::thread::spawn(move || handle_client(&mut stream));
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn handle_client(mut stream: &mut TcpStream) -> std::io::Result<()> {
    if let Ok(message) = ClientMessage::read_from(stream) {
        match message {
            ClientMessage::Connect {} => {
                println!("Recibí un connect: {:?}", message);
                let connack = BrokerMessage::Connack {
                    //session_present: true,
                    //return_code: 0,
                };
                println!("Sending connack: {:?}", connack);
                connack.write_to(&mut stream).unwrap();
            }
            _ => {
                println!("Recibí un mensaje que no es connect");
            }
        }
    } else {
        println!("Soy el broker y no pude leer el mensaje");
    }

    let cloned_stream = stream.try_clone()?; // Clone the TcpStream
    let reader = BufReader::new(cloned_stream); // Use the cloned stream in BufReader
    let mut lines = reader.lines();
    while let Some(Ok(line)) = lines.next() {
        println!("me llego un {:?}", line);
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
