use std::io::Error;

///  reason codes in HEX
const SUCCESS_HEX: u8 = 0x00;
const NO_MATCHING_SUBSCRIBERS_HEX: u8 = 0x10;
const UNSPECIFIED_ERROR_HEX: u8 = 0x80;
const IMPLEMENTATION_SPECIFIC_ERROR_HEX: u8 = 0x83;
const NOT_AUTHORIZED_HEX: u8 = 0x87;
const TOPIC_NAME_INVALID_HEX: u8 = 0x90;
const PACKET_ID_IN_USE_HEX: u8 = 0x91;
const QUOTA_EXCEEDED_HEX: u8 = 0x97;
const PAYLOAD_FORMAT_INVALID_HEX: u8 = 0x99;

pub enum ReasonCode {
    Success { reason_code: u8 },
    NoMatchingSubscribers { reason_code: u8 },
    UnspecifiedError { reason_code: u8 },
    ImplementationSpecificError { reason_code: u8 },
    NotAuthorized { reason_code: u8 },
    TopicNameInvalid { reason_code: u8 },
    PacketIdentifierInUse { reason_code: u8 },
    QuotaExceeded { reason_code: u8 },
    PayloadFormatInvalid { reason_code: u8 },
    
}

impl ReasonCode {
    pub fn new(&self, reason_code: u8) -> Result<ReasonCode, Error> {
        match reason_code {
            SUCCESS_HEX => Ok(ReasonCode::Success { reason_code }),
            NO_MATCHING_SUBSCRIBERS_HEX => {
                Ok(ReasonCode::NoMatchingSubscribers { reason_code })
            }
            UNSPECIFIED_ERROR_HEX => Ok(ReasonCode::UnspecifiedError { reason_code }),
            IMPLEMENTATION_SPECIFIC_ERROR_HEX => {
                Ok(ReasonCode::ImplementationSpecificError { reason_code })
            }
            NOT_AUTHORIZED_HEX => Ok(ReasonCode::NotAuthorized { reason_code }),
            TOPIC_NAME_INVALID_HEX => Ok(ReasonCode::TopicNameInvalid { reason_code }),
            PACKET_ID_IN_USE_HEX => Ok(ReasonCode::PacketIdentifierInUse { reason_code }),
            QUOTA_EXCEEDED_HEX => Ok(ReasonCode::QuotaExceeded { reason_code }),
            PAYLOAD_FORMAT_INVALID_HEX => {
                Ok(ReasonCode::PayloadFormatInvalid { reason_code })
            }
            _ => Err(Error::new(
                std::io::ErrorKind::Other,
                "Reason code inválido",
            )),
        }
    }
}
