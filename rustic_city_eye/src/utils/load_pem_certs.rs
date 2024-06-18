use std::{fs::File, io::BufReader, path::Path};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};

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

pub fn load_pem_key(path: &Path) -> Result<PrivateKeyDer<'static>, ProtocolError> {
    let certs_file = File::open(path).map_err(|_| ProtocolError::ReadingPEMCertsError)?;
    let mut reader = BufReader::new(certs_file);

    match rustls_pemfile::private_key(&mut reader) {
        Ok(key) => {
            if let Some(k) = key {
                return Ok(k);
            }
        }
        Err(_) => return Err(ProtocolError::ReadingPEMCertsError),
    };
    Err(ProtocolError::ReadingPEMCertsError)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_pem_certs;

    #[test]
    fn test_01_reading_pem_file_ok() -> Result<(), ProtocolError> {
        let path = Path::new("./src/mqtt/cert/ca_bundle.pem");
        let certs = load_pem_certs::load_pem_certs(path);

        assert!(certs.is_ok());

        Ok(())
    }

    #[test]
    fn test_02_reading_private_key_ok() -> Result<(), ProtocolError> {
        let path = Path::new("./src/mqtt/cert/private_key.pem");
        let private_key = load_pem_certs::load_pem_key(path);

        assert!(private_key.is_ok());

        Ok(())
    }
}
