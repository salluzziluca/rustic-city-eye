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
       
        let topic_names = vec!["incident", "drone_locations", "incident_resolved", "camera_update","attending_incident",
        "single_drone_disconnect"];
        
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


}