pub mod doc;
pub mod drone_system;
mod utils;
#[doc = include_str!("doc/informe.md")]
#[doc = include_str!("doc/marco_teorico.md")]

pub mod mqtt {
    pub mod connect_config;
    pub mod messages_config;
    pub mod publish_config;
    pub mod subscribe_config;

    pub mod broker_message;
    pub mod client;
    pub mod client_message;
    pub mod connect_properties;
    pub mod protocol_error;
    pub mod publish_properties;
    pub mod reason_code;
    pub mod subscribe_properties;
    pub mod topic;
    pub mod will_properties;

    pub mod broker;
    pub mod broker_config;

    pub mod client_return;
    pub mod error;
    pub mod protocol_return;
}

pub mod monitoring {
    pub mod incident;
    pub mod monitoring_app;
    pub mod monitoring_config;
}

pub mod surveilling {
    pub mod camera;
    pub mod camera_system;
    pub mod location;
}

// pub mod utils {
//     pub mod reader;
//     pub mod threadpool;
//     pub mod writer;
// }
