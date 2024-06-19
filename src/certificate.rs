fn load_encrypted_private_key() -> Result<PKey<Private>, String> {
    let mut file = File::open("key.pem").map_err(|err| err.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;

    PKey::private_key_from_pem_passphrase(&buffer, b"password").unwrap()
}

pub fn load_ssl_certificate(server_cfg: &ServerConfig) -> Result<(), String> {
    Ok(())
}
