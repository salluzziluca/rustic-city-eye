#[cfg(test)]
mod tests {
    use rustic_city_eye::monitoring::incident::Incident;
    use rustic_city_eye::mqtt::broker::Broker;
    use rustic_city_eye::mqtt::broker_message::BrokerMessage;
    use rustic_city_eye::mqtt::client_message;
    use rustic_city_eye::mqtt::publish::publish_properties::{PublishProperties, TopicProperties};
    use rustic_city_eye::mqtt::subscribe_properties::SubscribeProperties;
    use rustic_city_eye::mqtt::subscription::Subscription;
    use rustic_city_eye::mqtt::{
        client_message::ClientMessage, protocol_error::ProtocolError,
        protocol_return::ProtocolReturn,
    };
    use rustic_city_eye::utils::incident_payload;
    use rustic_city_eye::utils::{location::Location, payload_types::PayloadTypes};
    use rustls::{
        ClientConfig, ClientConnection, KeyLogFile, RootCertStore, ServerConnection, StreamOwned,
    };

    use std::fs::File;
    use std::io::{BufReader, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{mpsc, Arc};
    use std::thread;

    #[test]
    fn test_mensaje_invalido_da_error() -> Result<(), ProtocolError> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
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

            let (message_to_write_sender, _) = mpsc::channel();
            let stream_ref = Arc::clone(&stream);

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref);
                        println!("Message {:?} received", result);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_connect() -> Result<(), ProtocolError> {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let connect_config =
            client_message::Connect::read_connect_config("./src/monitoring/connect_config.json")
                .unwrap();

        let connect = ClientMessage::Connect(connect_config);

        thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
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
                            assert!(false, "Assertion failed: No se recibio un Connack");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_envio_connect_con_id_repetido_y_desconecta() -> Result<(), ProtocolError> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let connect_config =
            client_message::Connect::read_connect_config("./src/monitoring/connect_config.json")
                .unwrap();

        let connect = ClientMessage::Connect(connect_config);
        let second_connect = connect.clone();

        thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
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
                            assert!(false, "Assertion failed: No se recibio un Connack");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
                }
            }
        }

        thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
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

            second_connect.write_to(tls_stream.get_ref()).unwrap();
        });

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
                            assert!(false, "Assertion failed: No se recibio un Connack");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::DisconnectSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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
                            assert!(false, "Assertion failed: No se recibio un Suback");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::SubackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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
            let stream = TcpStream::connect(addr).unwrap();
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
                            assert!(false, "Assertion failed: No se recibio un Puback");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::PubackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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
            let stream = TcpStream::connect(addr).unwrap();
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

            thread::spawn(move || loop {
                if let Ok(message) = message_to_write_receiver.try_recv() {
                    match message {
                        _ => {
                            assert!(true);
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::NoAckSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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
                            assert!(false, "Assertion failed: No se recibio un Unsuback");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::UnsubackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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
                            assert!(false, "Assertion failed: No se recibio un Unsuback");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::DisconnectRecieved);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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

            thread::spawn(move || loop {
                if let Ok(message) = message_to_write_receiver.try_recv() {
                    match message {
                        BrokerMessage::Pingresp => assert!(true),
                        _ => {
                            assert!(false, "Assertion failed: No se recibio un Unsuback");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::PingrespSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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

        let connect_config = client_message::Connect::read_connect_config(
            "./tests/connect_config_test/config_with_invalid_auth_method.json",
        )
        .unwrap();

        let connect = ClientMessage::Connect(connect_config.clone());

        thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
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
                            assert!(false, "Assertion failed: No se recibio un Unsuback");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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
            client_message::Connect::read_connect_config("./src/monitoring/connect_config.json")
                .unwrap();

        let connect = ClientMessage::Connect(connect_config);
        let second_connect = connect.clone();

        thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
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
                            assert!(false, "Assertion failed: No se recibio un Connack");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
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
                            assert!(false, "Assertion failed: No se recibio un Unsuback");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("{:?}", result);
                        assert_eq!(result, ProtocolReturn::DisconnectRecieved);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
                }
            }
        }

        // vuelvo a conectar
        thread::spawn(move || {
            let stream = TcpStream::connect(addr).unwrap();
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
                            assert!(false, "Assertion failed: No se recibio un Connack");
                        }
                    }
                    break;
                }
            });

            loop {
                match ClientMessage::read_from(stream.get_ref()) {
                    Ok(message) => {
                        let result =
                            broker.handle_message(message, &message_to_write_sender, stream_ref)?;
                        println!("Message {:?} received", result);
                        assert_eq!(result, ProtocolReturn::ConnackSent);
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("Error {}", err);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
