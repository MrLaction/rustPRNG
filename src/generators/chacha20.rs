use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use num_bigint::BigUint;
use sha2::{Digest, Sha256};

use super::PseudoRandomGenerator;

pub struct ChaCha20Generator {
    cipher: ChaCha20,
}

impl ChaCha20Generator {
    pub fn new(key_seed: BigUint, nonce_seed: BigUint) -> Self {
        //Derive a 32-byte key from the numeric seed.
        let mut key_hasher = Sha256::new();
        key_hasher.update(b"workshop3/chacha20/key/v1");
        key_hasher.update(key_seed.to_bytes_be());

        let key: [u8; 32] = key_hasher.finalize().into();

        //Derive a 12-byte nonce using a separate domain.
        let mut nonce_hasher = Sha256::new();
        nonce_hasher.update(b"workshop3/chacha20/nonce/v1");
        nonce_hasher.update(nonce_seed.to_bytes_be());

        let nonce_digest = nonce_hasher.finalize();
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_digest[..12]);

        //The cipher tracks its position across successive calls.
        let cipher = ChaCha20::new(&key.into(), &nonce.into());

        Self { cipher }
    }
}

impl PseudoRandomGenerator for ChaCha20Generator {
    fn fill_bytes(&mut self, output: &mut [u8]) {
        //Applying the keystream to zeros produces raw pseudorandom bytes.
        output.fill(0);

        self.cipher
            .try_apply_keystream(output)
            .expect("ChaCha20 stream exhausted");
    }

    fn name(&self) -> &'static str {
        "ChaCha20"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::generate_40_digit_number;

    fn test_generator() -> ChaCha20Generator {
        ChaCha20Generator::new(
            BigUint::from(12345u64),
            BigUint::from(67890u64),
        )
    }

    #[test]
    fn generated_numbers_have_40_digits_and_are_reproducible() {
        let mut generator_a = test_generator();
        let mut generator_b = test_generator();

        for _ in 0..1000 {
            let a = generate_40_digit_number(&mut generator_a);
            let b = generate_40_digit_number(&mut generator_b);

            assert_eq!(a.to_string().len(), 40);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn changing_either_seed_changes_output() {
        let mut baseline = test_generator();
        let mut different_key = ChaCha20Generator::new(
            BigUint::from(12346u64),
            BigUint::from(67890u64),
        );
        let mut different_nonce = ChaCha20Generator::new(
            BigUint::from(12345u64),
            BigUint::from(67891u64),
        );

        let mut original = [0u8; 64];
        let mut changed_key = [0u8; 64];
        let mut changed_nonce = [0u8; 64];

        baseline.fill_bytes(&mut original);
        different_key.fill_bytes(&mut changed_key);
        different_nonce.fill_bytes(&mut changed_nonce);

        assert_ne!(original, changed_key);
        assert_ne!(original, changed_nonce);
    }

    #[test]
    fn output_is_independent_of_buffer_contents_and_chunk_sizes() {
        let mut whole_generator = test_generator();
        let mut split_generator = test_generator();

        let mut whole = [0u8; 128];
        let mut split = [0xffu8; 128];

        whole_generator.fill_bytes(&mut whole);

        //Use uneven chunks to cross internal cipher block boundaries.
        for chunk in split.chunks_mut(17) {
            split_generator.fill_bytes(chunk);
        }

        assert_eq!(whole, split);
    }
}