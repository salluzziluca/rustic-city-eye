use std::net::TcpStream;

use self::broker_message::BrokerMessage;
use self::client_message::ClientMessage;
use self::protocol_error::ProtocolError;

#[path = "broker_message.rs"]
mod broker_message;
#[path = "client_message.rs"]
mod client_message;
#[path = "protocol_error.rs"]
mod protocol_error;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(address: &str) -> Result<Client, ProtocolError> {
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let connect = ClientMessage::Connect {
            cleanStart: true,
            lastWillFlag: true,
            lastWillRetain: true,
            username: "prueba".to_string(),
            password: "".to_string(),
        };
        println!("Sending connect message to broker: {:?}", connect);
        connect.write_to(&mut stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut stream) {
            match message {
                BrokerMessage::Connack {
                   //session_present,
                    //return_code,
                } => {
                    println!("Recibí un connack: {:?}", message);
                }
            }
        } else {
            println!("soy el client y no pude leer el mensaje");
        }

        Ok(Client { stream })
    }

    pub fn publish_message(&mut self, message: &str) {
        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "juan".to_string(),
            qos: 0,
            retain_flag: true,
            payload: message.to_string(),
            dup_flag: true,
        };

        let _ = publish.write_to(&mut self.stream);
    }
}
