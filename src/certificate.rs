use openssl::{
    pkey::{PKey, Private},
    ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod},
};
use std::{fs::File, io::Read, path::Path};

fn load_encrypted_private_key(certificate_folder_path: &str) -> Result<PKey<Private>, String> {
    let path = Path::new(certificate_folder_path).join("key.pem");
    let mut file = File::open(path).map_err(|err| err.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;

    PKey::private_key_from_pem_passphrase(&buffer, b"password").map_err(|err| err.to_string())
}

pub fn load_ssl_certificate(certificate_folder_path: &str) -> Result<SslAcceptorBuilder, String> {
    let mut builder =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).map_err(|err| err.to_string())?;

    let pkey =
        load_encrypted_private_key(certificate_folder_path).map_err(|err| err.to_string())?;

    // set the encrypted private key
    builder
        .set_private_key(&pkey)
        .map_err(|err| err.to_string())?;

    // set the certificate chain file location
    builder
        .set_certificate_chain_file("cert.pem")
        .map_err(|err| err.to_string())?;

    Ok(builder)
}
