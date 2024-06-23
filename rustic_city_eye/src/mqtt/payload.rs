use std::{any::Any, fmt, io::Write};

use super::protocol_error::ProtocolError;

/// La API del cliente le permite a sus usuarios enviar packets del tipo Publish.
/// Dentro de este tipo de packets, hay un campo llamado Payload, que contiene el
/// application message que el usuario de la API quiere enviar.
///
/// La idea es que el usuario haga uso de este trait, de forma tal que el que implemente
/// este trait sea capaz de escribir y leer instancias de si mismo en un stream.
pub trait Payload {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), ProtocolError>;
    fn as_any(&self) -> &dyn Any;
}

impl fmt::Debug for dyn Payload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Payload trait object")
    }
}

impl dyn Payload {
    pub fn is<T: Any>(&self) -> bool {
        self.as_any().is::<T>()
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct TestPayload {
        pub value: u32,
    }

    impl Payload for TestPayload {
        fn write_to(&self, stream: &mut dyn Write) -> Result<(), ProtocolError> {
            let value_string = self.value.to_string();
            let _ = stream
                .write_all(value_string.as_bytes())
                .map_err(|_e| ProtocolError::WriteError);
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_payload_trait() {
        let payload = TestPayload { value: 42 };
        let mut cursor = Cursor::new(Vec::new());

        payload.write_to(&mut cursor).unwrap();

        let payload: &dyn Payload = &TestPayload { value: 42 };
        assert!(payload.is::<TestPayload>());
        assert_eq!(payload.downcast_ref::<TestPayload>().unwrap().value, 42);
    }
}
