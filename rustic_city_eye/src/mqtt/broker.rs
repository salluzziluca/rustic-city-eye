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
fn handle_client(mut stream: &mut TcpStream) -> std::io::Result<()> {
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
                //println!("Recibí un connect: {:?}", message);
                println!("Recibí un connect");
                let connack = BrokerMessage::Connack {
                    //session_present: true,
                    //return_code: 0,
                };
                println!("Sending connack: {:?}", connack);
                connack.write_to(&mut stream).unwrap();
            }
            ClientMessage::Publish {
                packet_id: _,
                topic_name: _,
                qos,
                retain_flag: _,
                payload: _,
                dup_flag: _,
                properties: _,
            } => {
                println!("Recibí un publish: {:?}", message);

                if qos == 1 {
                    println!("sending puback...");
                    let puback = BrokerMessage::Puback { reason_code: 1 };
                    puback.write_to(stream).unwrap();
                }
            }
            ClientMessage::Subscribe {
                packet_id: _,
                topic_name: _,
                properties: _,
            } => {
                println!("Recibí un subscribe: {:?}", message);
                let suback = BrokerMessage::Suback {
                    packet_id_msb: 0,
                    packet_id_lsb: 1,
                    reason_code: 0,
                };
                println!("Sending suback: {:?}", suback);
                suback.write_to(&mut stream).unwrap();
            }
            _ => {
                println!("Recibí un mensaje desconocido: {:?}", message);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Mensaje desconocido",
                ));
            }
        }
    }

    Ok(())
}
