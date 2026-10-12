use rand::rngs::OsRng;
use rsa::{Oaep, RsaPublicKey, traits::PublicKeyParts};
use sha2::Sha256;

pub struct Transmitter {
    public_key: RsaPublicKey,
}

impl Transmitter {
    pub fn new(public_key: RsaPublicKey) -> Self {
        Self { public_key }
    }

    pub fn max_message_bytes(&self) -> usize {
        //OAEP requires two SHA-256 digests and two additional bytes.
        self.public_key.size().saturating_sub(2 * 32 + 2)
    }

    pub fn encrypt(&self, message: &str) -> Result<Vec<u8>, String> {
        let bytes = message.as_bytes();
        let maximum = self.max_message_bytes();

        if bytes.len() > maximum {
            return Err(format!(
                "Message exceeds the limit: {} bytes provided, {maximum} allowed.",
                bytes.len()
            ));
        }

        let mut rng = OsRng;

        self.public_key
            .encrypt(&mut rng, Oaep::new::<Sha256>(), bytes)
            .map_err(|error| format!("Encryption failed: {error}"))
    }
}