pub mod doc;
pub mod utils;
#[doc = include_str!("doc/informe.md")]
#[doc = include_str!("doc/marco_teorico.md")]

pub mod mqtt {
    pub mod connect {
        pub mod connect_properties;
        pub mod last_will;
        pub mod will_properties;
    }
    pub mod publish {
        pub mod publish_config;
        pub mod publish_properties;
    }

    pub mod messages_config;
    pub mod subscribe_config;

    pub mod broker_message;
    pub mod client;
    pub mod client_message;
    pub mod connack_properties;
    pub mod protocol_error;
    pub mod reason_code;
    pub mod subscribe_properties;
    pub mod subscription;
    pub mod topic;

    pub mod broker;

    pub mod client_return;
    pub mod error;
    pub mod protocol_return;

    pub mod payload;
}

pub mod monitoring {
    pub mod incident;
    pub mod monitoring_app;
    pub mod monitoring_config;
}

pub mod surveilling {
    pub mod camera;
    pub mod camera_error;
    pub mod camera_system;
}

pub mod drones {
    pub mod drone;
    pub mod drone_center;
    pub mod drone_config;
    pub mod drone_state;
    pub mod drone_system;

    pub mod drone_error;
}
