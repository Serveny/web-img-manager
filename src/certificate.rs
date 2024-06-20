use openssl::{
    pkey::{PKey, Private},
    ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod},
};
use std::{fs::File, io::Read, path::Path};

fn load_encrypted_private_key(key_pem_path: &str) -> Result<PKey<Private>, String> {
    let key_pem_path = Path::new(key_pem_path)
        .canonicalize()
        .map_err(|err| format!("Can't build key.pem path: {err}"))?;
    let mut file = File::open(&key_pem_path).map_err(|err| {
        format!(
            "{}: {}",
            &key_pem_path.to_str().unwrap_or("no path"),
            err.to_string()
        )
    })?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;

    PKey::private_key_from_pem_passphrase(&buffer, b"password").map_err(|err| err.to_string())
}

pub fn load_ssl_certificate(
    cert_pem_path: &str,
    key_pem_path: &str,
) -> Result<SslAcceptorBuilder, String> {
    let cert_pem_path = Path::new(cert_pem_path)
        .canonicalize()
        .map_err(|err| format!("Can't build cert.pem path: {err}"))?;
    let cert_pem_path = cert_pem_path
        .to_str()
        .ok_or(String::from("cert.pem path to str failed"))?;

    let mut builder =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).map_err(|err| err.to_string())?;

    let pkey = load_encrypted_private_key(key_pem_path).map_err(|err| err.to_string())?;

    // set the encrypted private key
    builder
        .set_private_key(&pkey)
        .map_err(|err| err.to_string())?;

    // set the certificate chain file location
    builder
        .set_certificate_chain_file(cert_pem_path)
        .map_err(|err| err.to_string())?;

    Ok(builder)
}
