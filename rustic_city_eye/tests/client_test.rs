#[cfg(test)]
mod tests {
    use rustic_city_eye::{
        monitoring::incident::Incident,
        mqtt::{
            broker::Broker,
            broker_message::BrokerMessage,
            client::handle_message,
            client_return::ClientReturn,
            connack_properties::ConnackProperties,
            protocol_error::ProtocolError,
            publish::publish_properties::{PublishProperties, TopicProperties},
        },
        surveilling::camera_system::CameraSystem,
        utils::{
            incident_payload::IncidentPayload, location::Location, payload_types::PayloadTypes,
        },
    };
    use std::{
        io::Write,
        net::{TcpListener, TcpStream},
        sync::{mpsc::channel, Arc, Condvar, Mutex},
        thread,
    };

    #[test]
    fn test_recibir_connack() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

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

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            connack.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages, sender, tx, "juancito".to_string())
        }

        assert_eq!(result.unwrap(), ClientReturn::ConnackReceived);
    }

    #[test]
    fn test_recibir_puback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

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

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        let (sender, recibidor) = channel();
        let (tx, _) = channel();

        thread::spawn(move || loop {
            if let Ok(recibido) = recibidor.try_recv() {
                if recibido {
                    break;
                }
            }
        });
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages, sender, tx, "juancito".to_string())
        }

        assert_eq!(result.unwrap(), ClientReturn::PubackRecieved);
    }

    #[test]
    fn test_recibir_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let disconnect = BrokerMessage::Disconnect {
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
        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages, sender, tx, "juancito".to_string());
        }

        assert_eq!(result.unwrap(), ClientReturn::DisconnectRecieved);
    }

    #[test]
    fn test_recibir_suback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let suback = BrokerMessage::Suback {
            packet_id_msb: 3,
            packet_id_lsb: 1,
            reason_code: 3,
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            suback.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages, sender, tx, "juancito".to_string())
        }
        assert_eq!(result.unwrap(), ClientReturn::SubackRecieved);
    }

    #[test]
    fn test_recibir_publish_delivery() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

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
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            pub_delivery.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        //agregar id 1 a pending messages
        //pending_messages.push(1);
        let (sender, _) = channel();

        let (tx, _rx) = channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages.clone(), sender, tx, "juancito".to_string())
        }

        assert_eq!(result.unwrap(), ClientReturn::PublishDeliveryRecieved);
        //assert_eq!(pending_messages.len(), 0);
    }

    #[test]
    fn test_recibir_unsuback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let unsuback = BrokerMessage::Unsuback {
            packet_id_msb: 1,
            packet_id_lsb: 1,
            reason_code: 1,
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            unsuback.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages, sender, tx, "juancito".to_string())
        }

        assert_eq!(result.unwrap(), ClientReturn::UnsubackRecieved);
    }

    #[test]
    fn test_recibir_pingresp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let pingresp = BrokerMessage::Pingresp;
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            pingresp.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages, sender, tx, "juancito".to_string())
        }

        assert_eq!(result.unwrap(), ClientReturn::PingrespRecieved);
    }

    #[test]
    fn test_recibir_auth() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let auth = BrokerMessage::Auth {
            reason_code: 0x00_u8,
            authentication_method: "password-based".to_string(),
            authentication_data: vec![0x00_u8, 0x01_u8],
            reason_string: "success".to_string(),
            user_properties: vec![("juan".to_string(), "hola".to_string())],
        };

        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            auth.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error no especificado".to_string(),
        ));

        let pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, pending_messages, sender, tx, "juancito".to_string())
        }

        assert_eq!(result.unwrap(), ClientReturn::AuthRecieved);
    }

    #[test]
    fn test_creo_cliente_y_envio_sub_por_channel() {
        let args = vec!["127.0.0.1".to_string(), "5097".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }

        let _handle = thread::spawn(move || {
            let mut _camera_system = CameraSystem::with_real_client("127.0.0.1::5097".to_string());
        });
    }
}
