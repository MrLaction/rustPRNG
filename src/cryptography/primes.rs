use num_bigint::BigUint;
use rand::{RngCore, rngs::OsRng};

use crate::generators::PseudoRandomGenerator;
use crate::generators::chacha20::ChaCha20Generator;
use crate::prime::is_probably_prime;

const PRIME_BITS: usize = 1024;
const PRIME_BYTES: usize = PRIME_BITS / 8;
pub(super) const PUBLIC_EXPONENT: u32 = 65537;
const MAX_ATTEMPTS: usize = 100_000;

pub struct PrimePair {
    pub p: BigUint,
    pub q: BigUint,
    pub candidates_tested: usize,
}

fn create_generator() -> Result<ChaCha20Generator, String> {
    let mut entropy = [0u8; 64];
    let mut rng = OsRng;

    rng.try_fill_bytes(&mut entropy)
        .map_err(|error| format!("Could not obtain system randomness: {error}"))?;

    let key_seed = BigUint::from_bytes_be(&entropy[..32]);
    let nonce_seed = BigUint::from_bytes_be(&entropy[32..]);

    Ok(ChaCha20Generator::new(key_seed, nonce_seed))
}

fn generate_candidate(generator: &mut impl PseudoRandomGenerator) -> BigUint {
    let mut bytes = [0u8; PRIME_BYTES];
    generator.fill_bytes(&mut bytes);

    //Set the two highest bits so the product of two candidates has 2048 bits.
    bytes[0] |= 0b1100_0000;

    //Only odd candidates can be prime at this size.
    bytes[PRIME_BYTES - 1] |= 1;

    BigUint::from_bytes_be(&bytes)
}

pub fn generate_prime_pair() -> Result<PrimePair, String> {
    let mut generator = create_generator()?;
    let public_exponent = BigUint::from(PUBLIC_EXPONENT);
    let one = BigUint::from(1u32);

    //Avoid selecting factors that are too close to each other.
    let minimum_distance = &one << (PRIME_BITS - 100);

    let mut first_prime: Option<BigUint> = None;

    for candidates_tested in 1..=MAX_ATTEMPTS {
        let candidate = generate_candidate(&mut generator);

        //For prime e, this excludes factors whose predecessor is divisible by e.
        if &candidate % &public_exponent == one {
            continue;
        }

        if let Some(p) = &first_prime {
            let distance = if &candidate >= p {
                &candidate - p
            } else {
                p - &candidate
            };

            //This also rejects selecting the same factor twice.
            if distance <= minimum_distance {
                continue;
            }
        }

        if !is_probably_prime(&candidate) {
            continue;
        }

        match &first_prime {
            None => {
                first_prime = Some(candidate);
            }
            Some(p) => {
                return Ok(PrimePair {
                    p: p.clone(),
                    q: candidate,
                    candidates_tested,
                });
            }
        }
    }

    Err(format!(
        "Could not find two suitable probable primes after {MAX_ATTEMPTS} candidates."
    ))
}