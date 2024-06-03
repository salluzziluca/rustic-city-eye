use std::io::Error;

///  reason codes in HEX
pub const SUCCESS_HEX: u8 = 0x00;
pub const NO_MATCHING_SUBSCRIBERS_HEX: u8 = 0x10;
pub const UNSPECIFIED_ERROR_HEX: u8 = 0x80;
pub const IMPLEMENTATION_SPECIFIC_ERROR_HEX: u8 = 0x83;
pub const NOT_AUTHORIZED_HEX: u8 = 0x87;
pub const TOPIC_NAME_INVALID_HEX: u8 = 0x90;
pub const PACKET_ID_IN_USE_HEX: u8 = 0x91;
pub const QUOTA_EXCEEDED_HEX: u8 = 0x97;
pub const PAYLOAD_FORMAT_INVALID_HEX: u8 = 0x99;
pub const SUB_ID_DUP_HEX: u8 = 0x85;

#[derive(Debug, PartialEq)]
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
    SubIdDup { reason_code: u8 },
}

impl ReasonCode {
    pub fn new(reason_code: u8) -> Result<ReasonCode, Error> {
        match reason_code {
            SUCCESS_HEX => Ok(ReasonCode::Success { reason_code }),
            NO_MATCHING_SUBSCRIBERS_HEX => Ok(ReasonCode::NoMatchingSubscribers { reason_code }),
            UNSPECIFIED_ERROR_HEX => Ok(ReasonCode::UnspecifiedError { reason_code }),
            IMPLEMENTATION_SPECIFIC_ERROR_HEX => {
                Ok(ReasonCode::ImplementationSpecificError { reason_code })
            }
            NOT_AUTHORIZED_HEX => Ok(ReasonCode::NotAuthorized { reason_code }),
            TOPIC_NAME_INVALID_HEX => Ok(ReasonCode::TopicNameInvalid { reason_code }),
            PACKET_ID_IN_USE_HEX => Ok(ReasonCode::PacketIdentifierInUse { reason_code }),
            QUOTA_EXCEEDED_HEX => Ok(ReasonCode::QuotaExceeded { reason_code }),
            PAYLOAD_FORMAT_INVALID_HEX => Ok(ReasonCode::PayloadFormatInvalid { reason_code }),
            SUB_ID_DUP_HEX => Ok(ReasonCode::SubIdDup { reason_code }),
            _ => Err(Error::new(
                std::io::ErrorKind::Other,
                "Reason code inválido",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reason_code() {
        let reason_code = ReasonCode::new(SUCCESS_HEX);
        assert_eq!(
            reason_code.unwrap(),
            ReasonCode::Success { reason_code: 0x00 }
        );
    }

    #[test]
    fn test_new_reason_code_invalid() {
        let reason_code = ReasonCode::new(0x01);
        assert_eq!(
            reason_code.unwrap_err().to_string(),
            "Reason code inválido".to_string()
        );
    }
}
