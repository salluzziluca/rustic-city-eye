#[cfg(test)]
mod tests {
    use rustic_city_eye::monitoring::incident::Incident;
    use rustic_city_eye::mqtt::broker::Broker;
    use rustic_city_eye::mqtt::client_message::{self, Connect};
    use rustic_city_eye::mqtt::connect::connect_properties::ConnectProperties;
    use rustic_city_eye::mqtt::connect::will_properties::WillProperties;
    use rustic_city_eye::mqtt::publish::publish_properties::{PublishProperties, TopicProperties};
    use rustic_city_eye::mqtt::subscribe_properties::SubscribeProperties;
    use rustic_city_eye::mqtt::subscription::Subscription;
    use rustic_city_eye::mqtt::{
        client_message::ClientMessage, protocol_error::ProtocolError,
        protocol_return::ProtocolReturn,
    };
    use rustic_city_eye::utils::incident_payload;
    use rustic_city_eye::utils::{location::Location, payload_types::PayloadTypes};

    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    #[test]
    fn test_mensaje_invalido_da_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();

            stream.write_all("Hola".as_bytes()).unwrap();
        });

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
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

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
        if let Ok((stream, _)) = listener.accept() {
            result = match broker.handle_messages(stream) {
                Ok(r) => Ok(r),
                Err(e) => return Err(e),
            };
        }

        assert_eq!(result, Ok(ProtocolReturn::ConnackSent));

        Ok(())
    }

    // #[test]
    // fn test_envio_connect_con_id_repetido_y_desconecta() {
    //     let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    //     let addr = listener.local_addr().unwrap();

    //     let connect_config =
    //         client_message::Connect::read_connect_config("./src/monitoring/connect_config.json")
    //             .unwrap();

    //     let connect = ClientMessage::Connect(connect_config.clone());

    //     thread::spawn(move || {
    //         let mut stream = TcpStream::connect(addr).unwrap();
    //         let mut buffer = vec![];
    //         connect.write_to(&mut buffer).unwrap();
    //         stream.write_all(&buffer).unwrap();
    //     });

    //     let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

    //     if let Ok((stream, _)) = listener.accept() {
    //         let result = broker.handle_messages(stream);
    //         assert!(result.is_ok());
    //         assert_eq!(result.unwrap(), ProtocolReturn::ConnackSent);
    //     }

    
    //     // vuelvo a enviar el connect con el mismo id
    //     // let connect_propierties = ConnectProperties {
    //     //     session_expiry_interval: 1,
    //     //     receive_maximum: 2,
    //     //     maximum_packet_size: 10,
    //     //     topic_alias_maximum: 99,
    //     //     request_response_information: true,
    //     //     request_problem_information: false,
    //     //     user_properties: vec![
    //     //         ("Hola".to_string(), "Mundo".to_string()),
    //     //         ("Chau".to_string(), "Mundo".to_string()),
    //     //     ],
    //     //     authentication_method: "test".to_string(),
    //     //     authentication_data: vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8],
    //     // };
    //     // let will_properties = WillProperties::new(
    //     //     120,
    //     //     1,
    //     //     30,
    //     //     "plain".to_string(),
    //     //     "topic".to_string(),
    //     //     vec![1, 2, 3, 4, 5],
    //     //     vec![("propiedad".to_string(), "valor".to_string())],
    //     // );

    //     // let connect = Connect::new(
    //     //     true,
    //     //     true,
    //     //     1,
    //     //     true,
    //     //     35,
    //     //     connect_propierties,
    //     //     "monitoring_app".to_string(),
    //     //     will_properties,
    //     //     "topic".to_string(),
    //     //     "chauchis".to_string(),
    //     //     "prueba".to_string(),
    //     //     "".to_string(),
    //     // );

    //     let connect = ClientMessage::Connect(connect_config.clone());

    //     thread::spawn(move || {
    //         let mut stream = TcpStream::connect(addr).unwrap();
    //         let mut buffer = vec![];
    //         connect.write_to(&mut buffer).unwrap();
    //         stream.write_all(&buffer).unwrap();
    //     });

    //     if let Ok((stream_clone, _)) = listener.accept() {
    //         let result = broker.handle_messages(stream_clone);
    //         assert!(result.is_ok());
    //         assert_eq!(result.unwrap(), ProtocolReturn::DisconnectSent);
    //     }

    // }

    #[test]
    fn test_subscribe() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let properties =
            SubscribeProperties::new(1, vec![("propiedad".to_string(), "valor".to_string())]);

        let payload = Subscription::new("mensajes para juan".to_string(), "kvtr33".to_string());

        let sub = ClientMessage::Subscribe {
            packet_id: 1,
            properties,
            payload,
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            sub.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );
        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
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
            response_topic: "incident".to_string(),
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
            topic_name: "incident".to_string(),
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

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
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
            response_topic: "incident".to_string(),
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
            topic_name: "incident".to_string(),
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

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::NoAckSent);
    }

    #[test]
    fn test_unsubcribe() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let properties =
            SubscribeProperties::new(1, vec![("propiedad".to_string(), "valor".to_string())]);

        let payload = Subscription::new("mensajes para juan".to_string(), "kvtr33".to_string());

        let sub = ClientMessage::Unsubscribe {
            packet_id: 1,
            properties,
            payload,
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            sub.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
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
            client_id: "kvtr33".to_string(),
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            disconnect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
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

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::PingrespSent);
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

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::ConnackSent);

        Ok(())
    }

    #[test]
    fn connect_disconnect_connect() {
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

        let connect = Connect::new(
            true,
            true,
            1,
            true,
            35,
            connect_propierties,
            "kvtr33".to_string(),
            will_properties,
            "topic".to_string(),
            "chauchis".to_string(),
            "prueba".to_string(),
            "".to_string(),
        );

        let connect = ClientMessage::Connect(connect);

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );
        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::ConnackSent);

        // desconecto

        let disconnect = ClientMessage::Disconnect {
            reason_code: 1,
            session_expiry_interval: 1,
            reason_string: "desconecto_normal".to_string(),
            client_id: "kvtr33".to_string(),
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            disconnect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );
        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::DisconnectRecieved);

        // vuelvo a conectar
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

        let connect = Connect::new(
            true,
            true,
            1,
            true,
            35,
            connect_propierties,
            "kvtr33".to_string(),
            will_properties,
            "topic".to_string(),
            "chauchis".to_string(),
            "prueba".to_string(),
            "".to_string(),
        );

        let connect = ClientMessage::Connect(connect);

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connect.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ProtocolReturn, ProtocolError> = Err(
            ProtocolError::UnspecifiedError("Error no especificado".to_string()),
        );

        if let Ok((stream, _)) = listener.accept() {
            result = broker.handle_messages(stream);
        }

        assert_eq!(result.unwrap(), ProtocolReturn::ConnackSent);
    }
}
