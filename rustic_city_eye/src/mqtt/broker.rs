use std::env::args;
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
    while let Ok(message) = ClientMessage::read_from(stream) {
        match message {
            ClientMessage::Connect {} => {
                println!("Recibí un connect: {:?}", message);
                let connack = BrokerMessage::Connack {
                    //session_present: true,
                    //return_code: 0,
                };
                println!("Sending connack: {:?}", connack);
                connack.write_to(&mut stream).unwrap();
            },
            ClientMessage::Publish { packet_id: _, topic_name: _, qos: _, retain_flag: _, payload: _, dup_flag: _ } => {
                println!("Recibí un publish: {:?}", message);
                let puback = BrokerMessage::Puback {  };
                println!("sending puback {:?}", puback);
                puback.write_to(stream).unwrap();
            }
        }
    }
    Ok(())
}