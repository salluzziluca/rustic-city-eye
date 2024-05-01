use std::io::{BufWriter, Error, Read, Write};

struct connectProperties {
    sessionExpiryInterval: u32,
    receiveMaximum: u16,
    maximumPacketSize: u32,
    topicAliasMaximum: u16,
    requestResponseInformation: bool,
    requestProblemInformation: bool,
    userProperties: Vec<(String, String)>,
    authenticationMethod: String,
    authenticationData: Vec<u8>,
}

impl connectProperties {
    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), Error> {
        let mut writer = BufWriter::new(stream);

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<connectProperties, Error> {}
}
