#[cfg(test)]
mod tests {
    use rustic_city_eye::monitoring::incident::Incident;
    use rustic_city_eye::mqtt::client_message;
    use rustic_city_eye::mqtt::publish::publish_properties::{PublishProperties, TopicProperties};
    use rustic_city_eye::mqtt::subscribe_properties::SubscribeProperties;
    use rustic_city_eye::mqtt::{
        broker::handle_messages, client_message::ClientMessage, protocol_error::ProtocolError,
        protocol_return::ProtocolReturn,
    };
    use rustic_city_eye::utils::incident_payload;
    use rustic_city_eye::utils::{location::Location, payload_types::PayloadTypes};

    use std::collections::HashMap;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::{mpsc, Arc, RwLock};
    use std::thread;

    #[test]
    fn test_mensaje_invalido_da_error() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();

            stream.write_all("Hola".as_bytes()).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);
        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert!(result.is_err());
    }

    #[test]
    fn test_connect() -> Result<(), ProtocolError> {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let connect_config =
            client_message::Connect::read_connect_config("./src/monitoring/connect_config.json")
                .unwrap();

        let connect = ClientMessage::Connect(connect_config.clone());

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);
        let (id_sender, reciever) = mpsc::channel();

        thread::spawn(move || loop {
            if let Ok(reciever) = reciever.try_recv() {
                break;
            }
        });
        if let Ok((stream, _)) = listener.accept() {
            result = match handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            ) {
                Ok(r) => Ok(r),
                Err(e) => return Err(e),
            };
        }

        assert_eq!(result, Ok(ProtocolReturn::ConnackSent));

        Ok(())
    }

    #[test]
    fn test_envio_connect_con_id_repetido_y_desconecta() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let connect_config =
            client_message::Connect::read_connect_config("./src/monitoring/connect_config.json")
                .unwrap();

        let connect = ClientMessage::Connect(connect_config.clone());

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        //add an id to the clients_ids
        clients_ids
            .write()
            .unwrap()
            .insert("monitoring_app".to_string(), None);
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);
        let (id_sender, _) = mpsc::channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::DisconnectSent);
    }

    #[test]
    fn test_subscribe() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };

        let sub = ClientMessage::Subscribe {
            packet_id: 1,
            topic_name: "topico".to_string(),
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            sub.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);
        let (id_sender, _) = mpsc::channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::SubackSent);
    }
    #[test]
    fn test_publish() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };

        let properties = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );
        let location = Location::new(1.1, 1.12);
        let new = Incident::new(location);
        let incident_payload = incident_payload::IncidentPayload::new(new);
        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "mensajes para juan".to_string(),
            qos: 1,
            retain_flag: 1,
            payload: PayloadTypes::IncidentLocation(incident_payload),
            dup_flag: 0,
            properties,
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            publish.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::PubackSent);
    }

    #[test]
    fn test_publish_qos0() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };

        let properties = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );
        let location = Location::new(1.1, 1.12);
        let new = Incident::new(location);
        let incident_payload = incident_payload::IncidentPayload::new(new);
        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "mensajes para juan".to_string(),
            qos: 0,
            retain_flag: 1,
            payload: PayloadTypes::IncidentLocation(incident_payload),
            dup_flag: 0,
            properties,
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            publish.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::NoAckSent);
    }

    #[test]
    fn test_unsubcribe() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let unsub = ClientMessage::Unsubscribe {
            packet_id: 1,
            topic_name: "topico".to_string(),
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            unsub.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::UnsubackSent);
    }

    #[test]
    fn test_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let disconnect = ClientMessage::Disconnect {
            reason_code: 1,
            session_expiry_interval: 1,
            reason_string: "pasaron_cosas".to_string(),
            user_properties: vec![("propiedad".to_string(), "valor".to_string())],
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            disconnect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::DisconnectRecieved);
    }

    #[test]
    fn test_pingreq() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr: std::net::SocketAddr = listener.local_addr().unwrap();

        let pingreq = ClientMessage::Pingreq;

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            pingreq.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::PingrespSent);
    }

    #[test]
    fn test_auth() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let auth = ClientMessage::Auth {
            reason_code: 0,
            authentication_method: "password-based".to_string(),
            authentication_data: vec![],
            reason_string: "buendia".to_string(),
            user_properties: vec![("hola".to_string(), "mundo".to_string())],
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            auth.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::AuthRecieved);
    }

    #[test]
    fn test_auth_method_not_supported() -> Result<(), ProtocolError> {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let connect_config = client_message::Connect::read_connect_config(
            "./tests/connect_config_test/config_with_invalid_auth_method.json",
        )
        .unwrap();

        let connect = ClientMessage::Connect(connect_config.clone());

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids: Arc<
            RwLock<HashMap<String, Option<rustic_city_eye::mqtt::connect::last_will::LastWill>>>,
        > = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        let (id_sender, _) = mpsc::channel();

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::ConnackSent);

        Ok(())
    }
}
