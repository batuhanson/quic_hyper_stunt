use bytes::Bytes;
use quinn::RecvStream;
use rustls::{Certificate, PrivateKey};
use tracing::info;

use super::error::NetworkError;

pub fn generate_self_signed(
    subject_alt_names: Vec<String>,
) -> Result<(Certificate, PrivateKey), rcgen::RcgenError> {
    tracing::info!("generating self-signed certificate");
    let cert = rcgen::generate_simple_self_signed(subject_alt_names)?;
    let key = cert.serialize_private_key_der();
    Ok((
        rustls::Certificate(cert.serialize_der()?),
        rustls::PrivateKey(key),
    ))
}

pub async fn read_certs_key(
    cert_path: std::path::PathBuf,
    key_path: std::path::PathBuf,
) -> Result<(Certificate, rustls::PrivateKey), Box<dyn std::error::Error>> {
    let cert = std::fs::read(cert_path)?;

    let key = std::fs::read(key_path.clone())?;
    let key = if key_path.extension().map_or(false, |x| x == "der") {
        rustls::PrivateKey(key)
    } else {
        let pkcs8 = rustls_pemfile::pkcs8_private_keys(&mut &*key)?;
        match pkcs8.into_iter().next() {
            Some(x) => rustls::PrivateKey(x),
            None => {
                let rsa = rustls_pemfile::rsa_private_keys(&mut &*key)?;
                match rsa.into_iter().next() {
                    Some(x) => rustls::PrivateKey(x),
                    None => {
                        return Err("error reading key".into());
                    }
                }
            }
        }
    };
    Ok((Certificate(cert), key))
}

#[allow(unused)]
pub async fn bytes_escape(req: Bytes) -> Bytes {
    let mut escaped = Vec::new();
    for &x in req.iter() {
        let part = std::ascii::escape_default(x).collect::<Vec<_>>();
        escaped.extend(part);
    }
    info!("content = {:?}", String::from_utf8_lossy(&escaped));
    Bytes::from(escaped)
}

pub async fn read_recv_stream(
    mut recv: RecvStream,
    limit: Option<usize>,
) -> Result<Bytes, NetworkError> {
    let limit = limit.unwrap_or(64 * 1024);
    match recv.read_to_end(limit).await {
        Ok(req) => Ok(Bytes::from(req)),
        Err(e) => Err(NetworkError::RecvError(format!("{:?}", e.to_string()))),
    }
}