use std::{any::Any, fmt, io::Write};

/// La API del cliente le permite a sus usuarios enviar packets del tipo Publish.
/// Dentro de este tipo de packets, hay un campo llamado Payload, que contiene el
/// application message que el usuario de la API quiere enviar.
///
/// La idea es que el usuario haga uso de este trait, de forma tal que el que implemente
/// este trait sea capaz de escribir y leer instancias de si mismo en un stream.
pub trait Payload {
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()>;
    // fn read_from(&self, stream: &mut dyn Read) -> Result<PayloadTypes, Error>;
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
