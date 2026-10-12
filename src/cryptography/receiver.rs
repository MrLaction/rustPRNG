use rand::rngs::OsRng;
use rsa::{Oaep, RsaPrivateKey};
use sha2::Sha256;

pub struct Receiver {
    private_key: RsaPrivateKey,
}

impl Receiver {
    pub fn new(private_key: RsaPrivateKey) -> Self {
        Self { private_key }
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String, String> {
        let mut rng = OsRng;

        //Blinding randomizes the private-key operation internally.
        let plaintext = self
            .private_key
            .decrypt_blinded(
                &mut rng,
                Oaep::new::<Sha256>(),
                ciphertext,
            )
            .map_err(|_| "Decryption failed.".to_string())?;

        String::from_utf8(plaintext)
            .map_err(|_| "The recovered message is not valid UTF-8.".to_string())
    }
}