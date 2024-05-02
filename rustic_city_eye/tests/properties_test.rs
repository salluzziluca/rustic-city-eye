    use std::io::Cursor;
    use rustic_city_eye::mqtt::connect_propierties::ConnectProperties;


    #[test]
    fn test_connect_properties() {
        let mut buffer = Cursor::new(Vec::new());
        let connect_properties = ConnectProperties {
            session_expiry_interval: 1,
            receive_maximum: 2,
            maximum_packet_size: 10,
            topic_alias_maximum: 99,
            request_response_information: true,
            request_problem_information: false,
            user_properties: vec![("Hola".to_string(),"Mundo".to_string()), ("Chau".to_string(),"Mundo".to_string())],
            authentication_method: "test".to_string(),
            authentication_data: vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8],
        };

        connect_properties.write_to(&mut buffer).unwrap();
        buffer.set_position(0);

        let connect_properties_read = ConnectProperties::read_from(&mut buffer).unwrap();
        assert_eq!(connect_properties, connect_properties_read);
    }