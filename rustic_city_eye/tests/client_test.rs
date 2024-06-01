#[cfg(test)]
mod tests {
    use rustic_city_eye::{
        monitoring::incident::Incident,
        mqtt::{
            broker_message::BrokerMessage,
            client::handle_message,
            client_return::ClientReturn,
            protocol_error::ProtocolError,
            publish_properties::{PublishProperties, TopicProperties},
        },
        utils::{
            incident_payload::IncidentPayload, location::Location, payload_types::PayloadTypes,
        },
    };
    use std::{
        collections::HashMap,
        io::Write,
        net::{TcpListener, TcpStream},
        sync::{Arc, Mutex},
        thread,
    };
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

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError);

        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let pending_messages: Vec<u16> = Vec::new();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, subscriptions, pending_messages)
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

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError);

        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let pending_messages: Vec<u16> = Vec::new();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, subscriptions, pending_messages)
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
            sub_id: 2,
        };
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut buffer = vec![];
            suback.write_to(&mut buffer).unwrap();
            stream.write_all(&buffer).unwrap();
        });

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError);

        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let pending_messages: Vec<u16> = Vec::new();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, subscriptions, pending_messages)
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
        let location = Location::new("1".to_string(), "1".to_string());
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

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError);

        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let pending_messages: Vec<u16> = Vec::new();
        //agregar id 1 a pending messages
        //pending_messages.push(1);
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, subscriptions.clone(), pending_messages.clone())
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

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError);

        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let pending_messages: Vec<u16> = Vec::new();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, subscriptions, pending_messages)
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

        let mut result: Result<ClientReturn, ProtocolError> = Err(ProtocolError::UnspecifiedError);

        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let pending_messages: Vec<u16> = Vec::new();
        if let Ok((stream, _)) = listener.accept() {
            result = handle_message(stream, subscriptions, pending_messages)
        }

        assert_eq!(result.unwrap(), ClientReturn::PingrespRecieved);
    }
}
