#[doc = include_str!("doc/informe.md")]
#[doc = include_str!("doc/marco_teorico.md")]
pub mod camera_system;
pub mod doc;
pub mod drone_system;
pub mod monitoring_app;

pub mod mqtt {
    pub mod client;
    // pub mod broker;
    pub mod protocol_error;
    pub mod connect_properties;
    pub mod will_properties;
    pub mod writer;
    pub mod reader;
    pub mod subscribe_properties;
    pub mod client_message;
    pub mod broker_message;
    pub mod publish_properties;
    pub mod topic;

    pub mod neobroker;
    pub mod neoclient;

}
