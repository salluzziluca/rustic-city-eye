#[cfg(test)]
mod tests {
    use rustic_city_eye::mqtt::{
        broker_message::BrokerMessage, client::Client, client_return::ClientReturn,
        connect_config::ConnectConfig, connect_properties, protocol_error::ProtocolError,
        will_properties,
    };
    use std::{
        collections::HashMap,
        io::Write,
        net::{TcpListener, TcpStream},
        sync::{mpsc, Arc, Mutex},
        thread,
    };
    #[test]
    fn test_recibir_puback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (sender, receiver) = mpsc::channel();
        let address = "127.0.0.1::5000".to_string();
        let will_properties = will_properties::WillProperties::new(
            1,
            1,
            1,
            "a".to_string(),
            "a".to_string(),
            [1, 2, 3].to_vec(),
            vec![("a".to_string(), "a".to_string())],
        );

        let connect_properties = connect_properties::ConnectProperties::new(
            30,
            1,
            20,
            20,
            true,
            true,
            vec![("hola".to_string(), "chau".to_string())],
            "auth".to_string(),
            vec![1, 2, 3],
        );
        let connect_config = ConnectConfig::new(
            true,
            true,
            1,
            true,
            35,
            connect_properties,
            "juancito".to_string(),
            will_properties,
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            "user".to_string(),
            "pass".to_string(),
        );

        let cliente = Client::new(receiver, address, connect_config);

        let puback = BrokerMessage::Puback {
            packet_id_msb: 1,
            packet_id_lsb: 5,
            reason_code: 1,
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            puback.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError);

        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx): (mpsc::Sender<u16>, mpsc::Receiver<u16>) = mpsc::channel();
        if let Ok((stream, _)) = listener.accept() {
            result = Client::receive_messages(stream, rx, subscriptions)
        }

        assert_eq!(result.unwrap(), ClientReturn::PubackRecieved);
    }
}
