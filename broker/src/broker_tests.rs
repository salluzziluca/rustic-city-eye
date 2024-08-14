#[cfg(test)]
mod tests {

    use crate::broker::Broker;
    use protocol::topic::Topic;
    use utils::protocol_error::ProtocolError;

    use std::{collections::HashMap, env, fs::File, io::Cursor, path::PathBuf};

    #[test]
    fn test_01_getting_starting_topics_ok() -> Result<(), ProtocolError> {
        let project_dir = env!("CARGO_MANIFEST_DIR");

        let file_path = PathBuf::from(project_dir).join("src/config/topics");

        let topics = Broker::get_broker_starting_topics(file_path.to_str().unwrap())?;

        let topic_names = vec![
            "incident",
            "drone_locations",
            "incident_resolved",
            "camera_update",
            "attending_incident",
            "single_drone_disconnect",
        ];

        for topic_name in topic_names {
            assert!(topics.contains_key(topic_name));
        }

        Ok(())
    }

    #[test]
    fn test_02_reading_config_files_err() {
        let topics: Result<HashMap<String, Topic>, ProtocolError> =
            Broker::get_broker_starting_topics("./aca/estan/los/topics");
        let clients_auth_info = Broker::process_clients_file("./ahperoacavanlosclientesno");

        assert!(topics.is_err());
        assert!(clients_auth_info.is_err());
    }

    #[test]
    fn test_03_processing_set_of_args() -> Result<(), ProtocolError> {
        let args_ok = vec!["0.0.0.0".to_string(), "5000".to_string()];
        let args_err = vec!["este_port_abrira_tu_corazon".to_string()];

        let processing_good_args_result = Broker::process_starting_args(args_ok);
        let processing_bad_args_result = Broker::process_starting_args(args_err);

        assert!(processing_bad_args_result.is_err());
        assert!(processing_good_args_result.is_ok());

        let resulting_address = processing_good_args_result.unwrap();

        assert_eq!(resulting_address, "0.0.0.0:5000".to_string());

        Ok(())
    }

    #[test]
    fn test_04_processing_clients_auth_info_ok() -> Result<(), ProtocolError> {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        let file_path = PathBuf::from(project_dir).join("src/config/clients");

        let clients_auth_info = Broker::process_clients_file(file_path.to_str().unwrap())?;

        let file = match File::open(file_path.to_str().unwrap()) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingClientsFileError),
        };

        let expected_clients = Broker::read_clients_file(&file)?;

        for (client_id, _auth_info) in clients_auth_info {
            assert!(expected_clients.contains_key(&client_id));
        }

        Ok(())
    }

    #[test]
    fn test_05_processing_input_commands() -> Result<(), ProtocolError> {
        println!("starting test_05_processing_input_commands");

        let project_dir = env!("CARGO_MANIFEST_DIR");
        std::env::set_current_dir(project_dir).expect("Failed to set current directory");

        println!("Current directory: {:?}", env::current_dir().unwrap());

        let file_path = PathBuf::from(project_dir).join("src/config/topics");
        println!("file_path to process: {:?}", file_path);

        if !file_path.exists() {
            panic!("File does not exist: {:?}", file_path);
        }

        let mut broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()])?;
        let good_command = b"shutdown\n";
        let cursor_one = Cursor::new(good_command);

        let bad_command = b"apagate\n";
        let cursor_two = Cursor::new(bad_command);

        assert!(broker.process_input_command(cursor_one).is_ok());
        assert!(broker.process_input_command(cursor_two).is_err());

        Ok(())
    }

    mod tests2 {
        use std::{
            fs::File,
            io::{BufReader, Write},
            net::{TcpListener, TcpStream},
            path::PathBuf,
            sync::{mpsc, Arc},
            thread,
        };

        use protocol::{
            broker_message::BrokerMessage,
            client_message::{ClientMessage, Connect},
            protocol_return::ProtocolReturn,
            publish::{
                payload_types::PayloadTypes,
                publish_properties::{PublishProperties, TopicProperties},
            },
            subscribe::subscribe_properties::SubscribeProperties,
            subscription::Subscription,
        };
        use rustls::{
            ClientConfig, ClientConnection, KeyLogFile, RootCertStore, ServerConnection,
            StreamOwned,
        };
        use utils::{
            incident::Incident, incident_payload::IncidentPayload, location::Location,
            protocol_error::ProtocolError,
        };

        use crate::broker::Broker;

        #[test]
        fn test_mensaje_invalido_da_error() -> Result<(), ProtocolError> {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();

                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");

                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                tls_stream.get_ref().write_all("Hola".as_bytes()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);
                assert!(ClientMessage::read_from(stream.get_ref()).is_err());
            }
            Ok(())
        }

        #[test]
        fn test_connect() -> Result<(), ProtocolError> {
            // Set up a listener on a local port.
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();

            let connect_config =
                Connect::read_connect_config("monitoring_app/packets_config/connect_config.json")
                    .unwrap();

            let connect = ClientMessage::Connect(connect_config);

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                connect.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, rx) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Connack {
                                session_present: _,
                                reason_code,
                                properties: _,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Connack")
                            }
                        }
                        break;
                    }
                });

                thread::spawn(move || loop {
                    if rx.try_recv().is_ok() {
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn test_subscribe() -> Result<(), ProtocolError> {
            // Set up a listener on a local port.
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();

            let properties =
                SubscribeProperties::new(1, vec![("propiedad".to_string(), "valor".to_string())]);

            let payload = Subscription::new("incident".to_string(), "kvtr33".to_string());

            let sub = ClientMessage::Subscribe {
                packet_id: 1,
                properties,
                payload,
            };

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                sub.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Suback {
                                packet_id_msb: _,
                                packet_id_lsb: _,
                                reason_code,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Suback");
                            }
                        }
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::SubackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }

            Ok(())
        }

        #[test]
        fn test_publish() -> Result<(), ProtocolError> {
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
            let incident_payload = IncidentPayload::new(new);
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
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                publish.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Puback {
                                packet_id_msb: _,
                                packet_id_lsb: _,
                                reason_code,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Puback");
                            }
                        }
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::PubackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn test_publish_qos0() -> Result<(), ProtocolError> {
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
            let incident_payload = IncidentPayload::new(new);
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
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                publish.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if message_to_write_receiver.try_recv().is_ok() {
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::NoAckSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn test_unsubcribe() -> Result<(), ProtocolError> {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();

            let properties =
                SubscribeProperties::new(1, vec![("propiedad".to_string(), "valor".to_string())]);

            let payload = Subscription::new("incident".to_string(), "kvtr33".to_string());

            let unsub = ClientMessage::Unsubscribe {
                packet_id: 1,
                properties,
                payload,
            };

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                unsub.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Unsuback {
                                packet_id_msb: _,
                                packet_id_lsb: _,
                                reason_code,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Unsuback");
                            }
                        }
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::UnsubackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn test_disconnect() -> Result<(), ProtocolError> {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let disconnect = ClientMessage::Disconnect {
                reason_code: 1,
                session_expiry_interval: 1,
                reason_string: "pasaron_cosas".to_string(),
                client_id: "kvtr33".to_string(),
            };

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                disconnect.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Disconnect {
                                reason_code,
                                session_expiry_interval: _,
                                reason_string: _,
                                user_properties: _,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Unsuback");
                            }
                        }
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::DisconnectRecieved);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn test_pingreq() -> Result<(), ProtocolError> {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr: std::net::SocketAddr = listener.local_addr().unwrap();

            let pingreq = ClientMessage::Pingreq;

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                pingreq.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Pingresp => {}
                            _ => {
                                panic!("Assertion failed: No se recibio un Pingreq");
                            }
                        }
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::PingrespSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn test_auth_method_not_supported() -> Result<(), ProtocolError> {
            // Set up a listener on a local port.
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();

            let connect_config =
                Connect::read_connect_config("tests/src/config_with_invalid_auth_method.json")
                    .unwrap();

            let connect = ClientMessage::Connect(connect_config.clone());

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                connect.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Connack {
                                session_present: _,
                                reason_code,
                                properties: _,
                            } => {
                                assert_eq!(reason_code, 0x8C_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Connack");
                            }
                        }
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }
            Ok(())
        }

        #[test]
        fn connect_disconnect_connect() -> Result<(), ProtocolError> {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();

            let connect_config =
                Connect::read_connect_config("monitoring_app/packets_config/connect_config.json")
                    .unwrap();

            let connect = ClientMessage::Connect(connect_config);
            let second_connect = connect.clone();

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                connect.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, rx) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Connack {
                                session_present: _,
                                reason_code,
                                properties: _,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Connack");
                            }
                        }
                        break;
                    }
                });

                thread::spawn(move || loop {
                    if rx.try_recv().is_ok() {
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }

            // desconecto

            let disconnect = ClientMessage::Disconnect {
                reason_code: 1,
                session_expiry_interval: 1,
                reason_string: "desconecto_normal".to_string(),
                client_id: "monitoreo".to_string(),
            };

            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                disconnect.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();

            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);
                let (tx, _) = mpsc::channel();

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Disconnect {
                                reason_code,
                                session_expiry_interval: _,
                                reason_string: _,
                                user_properties: _,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Disconnect");
                            }
                        }
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::DisconnectRecieved);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }

            // vuelvo a conectar
            thread::spawn(move || {
                let stream = TcpStream::connect(addr).unwrap();
                let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
                let mut root_store = RootCertStore::empty();
                let project_dir = env!("CARGO_MANIFEST_DIR");
                let file_path = PathBuf::from(project_dir).join("broker/src/certs/cert.pem");
                println!("Test clients path: {:?}", file_path);
                let cert_file = &mut BufReader::new(File::open(file_path).unwrap());
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

                second_connect.write_to(tls_stream.get_ref()).unwrap();
            });

            let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
            if let Ok((stream, _)) = listener.accept() {
                let server_connection = match ServerConnection::new(broker.server_config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(ProtocolError::StreamError);
                    }
                };

                let tls_stream = StreamOwned::new(server_connection, stream);
                let stream = Arc::new(tls_stream);
                let (tx, rx) = mpsc::channel();

                let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                let stream_ref = Arc::clone(&stream);

                thread::spawn(move || loop {
                    if let Ok(message) = message_to_write_receiver.try_recv() {
                        match message {
                            BrokerMessage::Connack {
                                session_present: _,
                                reason_code,
                                properties: _,
                            } => {
                                assert_eq!(reason_code, 0x00_u8);
                            }
                            _ => {
                                panic!("Assertion failed: No se recibio un Connack");
                            }
                        }
                        break;
                    }
                });

                thread::spawn(move || loop {
                    if rx.try_recv().is_ok() {
                        break;
                    }
                });

                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result = broker.handle_message(
                            message,
                            &message_to_write_sender,
                            stream_ref,
                            tx,
                        )?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                    }
                }
            }

            Ok(())
        }
    }
}
