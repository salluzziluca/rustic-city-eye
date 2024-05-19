pub mod doc;
pub mod drone_system;
pub mod monitoring;
#[doc = include_str!("doc/informe.md")]
#[doc = include_str!("doc/marco_teorico.md")]
pub mod surveilling;

pub mod mqtt {
    pub mod client;
    pub mod broker_message;
    pub mod client_message;
    pub mod connect_properties;
    pub mod protocol_error;
    pub mod publish_properties;
    pub mod reader;
    pub mod subscribe_properties;
    pub mod topic;
    pub mod will_properties;
    pub mod writer;

    pub mod broker;
    pub mod broker_config;

    pub mod error;
}
