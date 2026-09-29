use num_bigint::BigUint;
use sha2::{Digest, Sha256};

use super::PseudoRandomGenerator;

pub struct Sha256Counter {
    seed: BigUint,
    salt: BigUint,
    counter: u64,
}

impl Sha256Counter {
    pub fn new(seed: BigUint, salt: BigUint) -> Self {
        Self {
            seed,
            salt,
            counter: 0,
        }
    }

    fn next_block(&mut self) -> [u8; 32] {
        let seed_bytes = self.seed.to_bytes_be();
        let salt_bytes = self.salt.to_bytes_be();

        let mut hasher = Sha256::new();

        //Domain identifier that defines this generator construction.
        hasher.update(b"workshop3/sha256-counter/v1");

        //Include lengths so different (seed, salt) pairs cannot produce
        //the same byte concatenation due to ambiguous boundaries.
        hasher.update((seed_bytes.len() as u64).to_be_bytes());
        hasher.update(&seed_bytes);

        hasher.update((salt_bytes.len() as u64).to_be_bytes());
        hasher.update(&salt_bytes);

        hasher.update(self.counter.to_be_bytes());

        let digest = hasher.finalize();
        self.counter = self.counter.wrapping_add(1);

        digest.into()
    }
}

impl PseudoRandomGenerator for Sha256Counter {
    fn fill_bytes(&mut self, output: &mut [u8]) {
        let mut written = 0;

        while written < output.len() {
            let block = self.next_block();
            let remaining = output.len() - written;
            let amount = remaining.min(block.len());

            output[written..written + amount].copy_from_slice(&block[..amount]);
            written += amount;
        }
    }

    fn name(&self) -> &'static str {
        "SHA-256 Counter"
    }
}
