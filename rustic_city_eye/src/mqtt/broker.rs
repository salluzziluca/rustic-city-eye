use std::env::args;
use std::net::{TcpListener, TcpStream};

use rustic_city_eye::mqtt::broker_message::BrokerMessage;
use rustic_city_eye::mqtt::client_message::ClientMessage;

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
#[allow(dead_code)]
fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    while let Ok(message) = ClientMessage::read_from(stream) {
        match message {
            ClientMessage::Connect {
                clean_start: _,
                last_will_flag: _,
                last_will_qos: _,
                last_will_retain: _,
                username: _,
                password: _,
                keep_alive: _,
                properties: _,
                client_id: _,
                will_properties: _,
                last_will_topic: _,
                last_will_message: _,
            } => {
                println!("Recibí un connect: {:?}", message);
                let connack = BrokerMessage::Connack {
                    //session_present: true,
                    //return_code: 0,
                };
                println!("Sending connack: {:?}", connack);
                connack.write_to(stream).unwrap();
            }
            ClientMessage::Publish {
                packet_id,
                topic_name: _,
                qos,
                retain_flag: _,
                payload: _,
                dup_flag: _,
                properties: _,
            } => {
                println!("Recibí un publish: {:?}", message);
                let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                if qos == 1 {
                    println!("sending puback...");
                    let puback = BrokerMessage::Puback { packet_id_msb: packet_id_bytes[0], packet_id_lsb: packet_id_bytes[1], reason_code: 1 };
                    puback.write_to(stream).unwrap();
                }
            }
        }
    }
    Ok(())
}
