
use std::{fs::File, io::BufReader, path::Path};

use rustls::pki_types::CertificateDer;

use crate::mqtt::protocol_error::ProtocolError;

/// lee las certificaciones PEM(contiene los certificados digitales y claves publicas
/// y privadas del protocolo).
pub fn load_pem_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, ProtocolError> {
    let certs_file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(ProtocolError::ReadingPEMCertsError),
    };

    let mut reader = BufReader::new(certs_file);

    rustls_pemfile::certs(&mut reader)
        .map(|result| match result {
            Ok(der) => Ok(der),
            Err(_) => Err(ProtocolError::ReadingPEMCertsError),
        })
        .collect()
}
