#[cfg(test)]
mod tests {
    use rustic_city_eye::{
        monitoring::incident::Incident,
        mqtt::{
            broker_message::BrokerMessage,
            client::Client,
            client_return::ClientReturn,
            connack_properties::ConnackProperties,
            protocol_error::ProtocolError,
            publish::publish_properties::{PublishProperties, TopicProperties},
        },
        utils::{
            incident_payload::IncidentPayload, location::Location, payload_types::PayloadTypes,
        },
    };
    use rustls::{
        ClientConfig, ClientConnection, KeyLogFile, RootCertStore, ServerConfig, ServerConnection,
        StreamOwned,
    };
    use rustls_pemfile::{certs, private_key};
    use std::{
        fs::File,
        io::BufReader,
        net::{TcpListener, TcpStream},
        sync::{
            mpsc,
            Arc,
        },
        thread,
    };

    #[test]
    fn test_recibir_connack() -> Result<(), ProtocolError> {
        let properties = ConnackProperties {
            session_expiry_interval: 0,
            receive_maximum: 0,
            maximum_packet_size: 0,
            topic_alias_maximum: 0,
            user_properties: vec![],
            authentication_method: "password-based".to_string(),
            authentication_data: vec![],
            assigned_client_identifier: "none".to_string(),
            maximum_qos: true,
            reason_string: "buendia".to_string(),
            wildcard_subscription_available: false,
            subscription_identifier_available: false,
            shared_subscription_available: false,
            server_keep_alive: 0,
            response_information: "none".to_string(),
            server_reference: "none".to_string(),
            retain_available: false,
        };

        let connack = BrokerMessage::Connack {
            session_present: true,
            reason_code: 0x00_u8,
            properties,
        };

        let listener = TcpListener::bind("127.0.0.1:5000").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(connack.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5000").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let (puback_notify_sender, _) = mpsc::channel();
                let (sender_channel, _) = mpsc::channel();
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::ConnackReceived);
                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_recibir_puback() -> Result<(), ProtocolError> {
        let puback = BrokerMessage::Puback {
            packet_id_msb: 1,
            packet_id_lsb: 5,
            reason_code: 1,
        };

        let listener = TcpListener::bind("127.0.0.1:5002").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(puback.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5002").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            let (puback_notify_sender, puback_notify_receiver) = mpsc::channel();
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let (sender_channel, _) = mpsc::channel();
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::PubackRecieved);
                break;
            }

            thread::spawn(move || {
                loop {
                    if puback_notify_receiver.try_recv().is_ok() {
                        break;
                    }
                }
            });
        }
        Ok(())
    }

    #[test]
    fn test_recibir_disconnect() -> Result<(), ProtocolError> {
        let disconnect = BrokerMessage::Disconnect {
            reason_code: 1,
            session_expiry_interval: 1,
            reason_string: "pasaron_cosas".to_string(),
            user_properties: vec![("propiedad".to_string(), "valor".to_string())],
        };
        
        let listener = TcpListener::bind("127.0.0.1:5003").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(disconnect.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5003").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            let (puback_notify_sender, _) = mpsc::channel();
            let (sender_channel, receive_channel) = mpsc::channel();
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::DisconnectRecieved);
                break;
            }

            thread::spawn(move || {
                loop {
                    if receive_channel.try_recv().is_ok() {
                        break;
                    }
                }
            });
        }

        Ok(())
    }

    #[test]
    fn test_recibir_suback() -> Result<(), ProtocolError> {
        let suback = BrokerMessage::Suback {
            packet_id_msb: 3,
            packet_id_lsb: 1,
            reason_code: 3,
        };
        
        let listener = TcpListener::bind("127.0.0.1:5004").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(suback.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5004").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            let (puback_notify_sender, _) = mpsc::channel();
            let (sender_channel, _) = mpsc::channel();
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::SubackRecieved);
                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_recibir_publish_delivery() -> Result<(), ProtocolError> {
        let topic = TopicProperties {
            topic_alias: 1,
            response_topic: "topic".to_string(),
        };

        let publish_propreties = PublishProperties {
            payload_format_indicator: 1,
            message_expiry_interval: 2,
            topic_properties: topic,
            correlation_data: vec![1, 2, 3],
            user_property: "propiedad".to_string(),
            subscription_identifier: 3,
            content_type: "content".to_string(),
        };
        let location = Location::new(1.1, 1.12);
        let new = Incident::new(location);
        let incident_payload = IncidentPayload::new(new);
        let pub_delivery = BrokerMessage::PublishDelivery {
            packet_id: 1,
            topic_name: "topic".to_string(),
            qos: 1,
            retain_flag: 2,
            payload: PayloadTypes::IncidentLocation(incident_payload),
            dup_flag: 4,
            properties: publish_propreties,
        };
        
        let listener = TcpListener::bind("127.0.0.1:5005").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(pub_delivery.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5005").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            let (puback_notify_sender, _) = mpsc::channel();
            let (sender_channel, receiver_channel) = mpsc::channel();
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::PublishDeliveryRecieved);
                break;
            }

            thread::spawn(move || {
                loop {
                    if receiver_channel.try_recv().is_ok() {
                        break;
                    }
                }
            });
        }

        Ok(())
    }

    #[test]
    fn test_recibir_unsuback() -> Result<(), ProtocolError> {
        let unsuback = BrokerMessage::Unsuback {
            packet_id_msb: 1,
            packet_id_lsb: 1,
            reason_code: 1,
        };
        
        let listener = TcpListener::bind("127.0.0.1:5001").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(unsuback.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5001").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            let (puback_notify_sender, _) = mpsc::channel();
            let (sender_channel, _) = mpsc::channel();
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::UnsubackRecieved);
                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_recibir_pingresp() -> Result<(), ProtocolError> {
        let pingresp = BrokerMessage::Pingresp;
        
        let listener = TcpListener::bind("127.0.0.1:5006").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(pingresp.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5006").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            let (puback_notify_sender, _) = mpsc::channel();
            let (sender_channel, _) = mpsc::channel();
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::PingrespRecieved);
                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_recibir_auth() -> Result<(), ProtocolError> {
        let auth = BrokerMessage::Auth {
            reason_code: 0x00_u8,
            authentication_method: "password-based".to_string(),
            authentication_data: vec![0x00_u8, 0x01_u8],
            reason_string: "success".to_string(),
            user_properties: vec![("juan".to_string(), "hola".to_string())],
        };

        let listener = TcpListener::bind("127.0.0.1:5007").unwrap();
        thread::spawn(move || {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            let certs = certs(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/cert.pem").unwrap(),
            ))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let private_key = private_key(&mut BufReader::new(
                &mut File::open("./src/mqtt/certs/private_key.pem").unwrap(),
            ))
            .unwrap()
            .unwrap();

            let server_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)
                .unwrap();

            let server_config = Arc::new(server_config);

            loop {
                if let Ok((stream, _)) = listener.accept() {
                    let server_connection = match ServerConnection::new(server_config.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };

                    let tls_stream = StreamOwned::new(server_connection, stream);
                    let stream = Arc::new(tls_stream);

                    assert!(auth.write_to(stream.get_ref()).is_ok());
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:5007").unwrap();
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let cert_file = &mut BufReader::new(File::open("./src/mqtt/certs/cert.pem").unwrap());
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).expect("me rompi");

        let tls_stream = StreamOwned::new(conn, stream);
        let tls_stream = Arc::new(tls_stream);

        loop {
            let (puback_notify_sender, _) = mpsc::channel();
            let (sender_channel, _) = mpsc::channel();
            if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
                let client_id = "client_test".to_string();
                let pending_messages = Vec::new();
                let result = Client::handle_message(
                    message,
                    pending_messages,
                    puback_notify_sender,
                    sender_channel,
                    client_id,
                )
                .unwrap();
                assert_eq!(result, ClientReturn::AuthRecieved);
                break;
            }
        }

        Ok(())
    }
}
