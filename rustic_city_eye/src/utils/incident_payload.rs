use crate::{
    monitoring::incident::Incident, mqtt::protocol_error::ProtocolError, utils::writer::write_u8,
};

#[derive(Clone, Debug, PartialEq)]
pub struct IncidentPayload {
    id: u8,
    incident: Incident,
}

/// Aqui se define el payload para los packets del tipo publish
/// que notifiquen de incidentes.
/// El IncidentPayload se identifica con un id = 1: este id sirve para
/// diferenciar los distintos tipos de payloads que va a tener la aplicacion.
impl IncidentPayload {
    pub fn new(incident: Incident) -> IncidentPayload {
        IncidentPayload { id: 1, incident }
    }

    ///Se sabe escribir sobre un stream dado.
    pub fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> Result<(), ProtocolError> {
        write_u8(stream, &self.id)?;

        self.incident.write_to(stream)?;
        Ok(())
    }
}
