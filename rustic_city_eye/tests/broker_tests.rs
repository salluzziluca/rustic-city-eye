#[cfg(test)]
mod tests {
    use rustic_city_eye::monitoring::incident::Incident;
    use rustic_city_eye::mqtt::connect_properties::ConnectProperties;
    use rustic_city_eye::mqtt::publish_properties::{PublishProperties, TopicProperties};
    use rustic_city_eye::mqtt::subscribe_properties::SubscribeProperties;
    use rustic_city_eye::mqtt::will_properties::WillProperties;
    use rustic_city_eye::mqtt::{
        broker::handle_messages, broker::Broker, client_message::ClientMessage,
        protocol_error::ProtocolError, protocol_return::ProtocolReturn,
    };
    use rustic_city_eye::utils::incident_payload;
    use rustic_city_eye::utils::{location::Location, payload_types::PayloadTypes};

    use std::collections::HashMap;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, RwLock};
    use std::thread;

    #[test]
    fn test_mensaje_invalido_da_error() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();

            stream.write_all(&"Hola".as_bytes()).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(vec![]);

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(stream, topics, packets, subs, clients_ids);
        }

        assert!(result.is_err());
    }
    #[test]
    fn test_connect() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let connect_propierties = ConnectProperties {
            session_expiry_interval: 1,
            receive_maximum: 2,
            maximum_packet_size: 10,
            topic_alias_maximum: 99,
            request_response_information: true,
            request_problem_information: false,
            user_properties: vec![
                ("Hola".to_string(), "Mundo".to_string()),
                ("Chau".to_string(), "Mundo".to_string()),
            ],
            authentication_method: "test".to_string(),
            authentication_data: vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8],
        };
        let will_properties = WillProperties::new(
            120,
            1,
            30,
            "plain".to_string(),
            "topic".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![("propiedad".to_string(), "valor".to_string())],
        );
        let connect = ClientMessage::Connect {
            clean_start: true,
            last_will_flag: true,
            last_will_qos: 1,
            last_will_retain: true,
            keep_alive: 35,
            properties: connect_propierties,
            client_id: "kvtr33".to_string(),
            will_properties,
            last_will_topic: "topic".to_string(),
            last_will_message: "chauchis".to_string(),
            username: "prueba".to_string(),
            password: "".to_string(),
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(vec![]);

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(stream, topics, packets, subs, clients_ids);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::ConnackSent);
    }
    #[test]
    fn test_envio_connect_con_id_repetido_y_desconecta() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let connect_propierties = ConnectProperties {
            session_expiry_interval: 1,
            receive_maximum: 2,
            maximum_packet_size: 10,
            topic_alias_maximum: 99,
            request_response_information: true,
            request_problem_information: false,
            user_properties: vec![
                ("Hola".to_string(), "Mundo".to_string()),
                ("Chau".to_string(), "Mundo".to_string()),
            ],
            authentication_method: "test".to_string(),
            authentication_data: vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8],
        };
        let will_properties = WillProperties::new(
            120,
            1,
            30,
            "plain".to_string(),
            "topic".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![("propiedad".to_string(), "valor".to_string())],
        );
        let connect = ClientMessage::Connect {
            clean_start: true,
            last_will_flag: true,
            last_will_qos: 1,
            last_will_retain: true,
            keep_alive: 35,
            properties: connect_propierties,
            client_id: "kvtr33".to_string(),
            will_properties,
            last_will_topic: "topic".to_string(),
            last_will_message: "chauchis".to_string(),
            username: "prueba".to_string(),
            password: "".to_string(),
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(vec!["kvtr33".to_string()]);

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(
                stream,
                topics.clone(),
                packets.clone(),
                subs.clone(),
                clients_ids.clone(),
            );
        }

        assert_eq!(result.unwrap(), ProtocolReturn::DisconnectSent);
    }

    #[test]
    fn test_subscribe() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let topic_properties = TopicProperties {
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
        let clients_ids = Arc::new(vec![]);

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(stream, topics, packets, subs, clients_ids);
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
        let location = Location::new("1".to_string(), "1".to_string());
        let new = Incident::new(location);
        let incident_payload = incident_payload::IncidentPayload::new(new);
        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "mensajes para juan".to_string(),
            qos: 1,
            retain_flag: 1,
            payload: PayloadTypes::IncidentLocation(incident_payload),
            dup_flag: 1,
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
        let clients_ids = Arc::new(vec![]);

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(stream, topics, packets, subs, clients_ids);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::PubackSent);
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
        let clients_ids = Arc::new(vec![]);

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(stream, topics, packets, subs, clients_ids);
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
        let clients_ids = Arc::new(vec![]);

        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(stream, topics, packets, subs, clients_ids);
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
        let clients_ids = Arc::new(vec![]);
        let mut result: Result<ProtocolReturn, ProtocolError> =
            Err(ProtocolError::UnspecifiedError);

        if let Ok((stream, _)) = listener.accept() {
            result = handle_messages(stream, topics, packets, subs, clients_ids);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::PingrespSent);
    }
}
