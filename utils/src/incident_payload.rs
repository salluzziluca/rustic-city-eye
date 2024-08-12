use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{incident::Incident, protocol_error::ProtocolError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IncidentPayload {
    pub id: u8,
    pub incident: Incident,
}

/// Aqui se define el payload para los packets del tipo publish
/// que notifiquen de incidentes.
/// El IncidentPayload se identifica con un id = 1: este id sirve para
/// diferenciar los distintos tipos de payloads que va a tener la aplicacion.
impl IncidentPayload {
    pub fn new(incident: Incident) -> IncidentPayload {
        let mut rng = rand::thread_rng();
        let id = rng.gen_range(0..255);
        IncidentPayload { id, incident }
    }

    ///Se sabe escribir sobre un stream dado.
    pub fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> Result<(), ProtocolError> {
        self.incident.write_to(stream)?;
        Ok(())
    }

    pub fn get_incident(&self) -> Incident {
        self.incident.clone()
    }
}
