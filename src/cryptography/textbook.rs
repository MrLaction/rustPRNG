use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};

use super::verified::VerifiedBatch;

pub struct Transmitter {
    n: BigUint,
    e: BigUint,
}

pub struct Receiver {
    n: BigUint,
    d: BigUint,
}

fn gcd(mut a: BigUint, mut b: BigUint) -> BigUint {
    while !b.is_zero() {
        let remainder = &a % &b;
        a = b;
        b = remainder;
    }

    a
}

fn modular_inverse(value: &BigUint, modulus: &BigUint) -> Option<BigUint> {
    let modulus = BigInt::from(modulus.clone());

    let mut old_remainder = modulus.clone();
    let mut remainder = BigInt::from(value.clone());

    let mut old_coefficient = BigInt::zero();
    let mut coefficient = BigInt::one();

    //Track the coefficient of value through the extended Euclidean algorithm.
    while !remainder.is_zero() {
        let quotient = &old_remainder / &remainder;

        let next_remainder = &old_remainder - &quotient * &remainder;
        old_remainder = remainder;
        remainder = next_remainder;

        let next_coefficient = &old_coefficient - &quotient * &coefficient;
        old_coefficient = coefficient;
        coefficient = next_coefficient;
    }

    if old_remainder != BigInt::one() {
        return None;
    }

    //Normalize a possibly negative coefficient into [0, modulus).
    let inverse = ((old_coefficient % &modulus) + &modulus) % &modulus;

    inverse.to_biguint()
}

fn ciphertext_width(n: &BigUint) -> usize {
    n.bits().div_ceil(8) as usize
}

fn payload_capacity(n: &BigUint) -> usize {
    //Reserve one byte for the nonzero prefix.
    ((n.bits() - 1) / 8) as usize - 1
}

pub fn build_participants(
    batch: &VerifiedBatch,
    p_index: usize,
    q_index: usize,
) -> Result<(Transmitter, Receiver), String> {
    let find = |index: usize| {
        batch
            .primes
            .iter()
            .find(|entry| entry.index == index)
            .ok_or_else(|| format!("Index {index} is not a verified prime."))
    };

    let p = &find(p_index)?.number;
    let q = &find(q_index)?.number;

    if p == q {
        return Err("Select two different prime values.".to_string());
    }

    if p.to_string().len() != 40 || q.to_string().len() != 40 {
        return Err("Both factors must have exactly 40 decimal digits.".to_string());
    }

    let one = BigUint::one();
    let p_minus_one = p - &one;
    let q_minus_one = q - &one;

    let common_divisor = gcd(
        p_minus_one.clone(),
        q_minus_one.clone(),
    );

    //lcm(a, b) = (a / gcd(a, b)) * b.
    let lambda = (&p_minus_one / common_divisor) * &q_minus_one;

    let e = BigUint::from(65537u32);
    let d = modular_inverse(&e, &lambda).ok_or_else(|| {
        "The factors are incompatible with e = 65537. Select another pair."
            .to_string()
    })?;

    let n = p * q;

    Ok((
        Transmitter {
            n: n.clone(),
            e,
        },
        Receiver { n, d },
    ))
}

impl Transmitter {
    pub fn modulus(&self) -> &BigUint {
        &self.n
    }

    pub fn exponent(&self) -> &BigUint {
        &self.e
    }

    pub fn block_capacity(&self) -> usize {
        payload_capacity(&self.n)
    }

    pub fn encrypt_number(&self, message: &BigUint) -> Result<BigUint, String> {
        if message >= &self.n {
            return Err("The message integer must be smaller than n.".to_string());
        }

        Ok(message.modpow(&self.e, &self.n))
    }

    pub fn encrypt_text(&self, message: &str) -> Result<Vec<u8>, String> {
        let width = ciphertext_width(&self.n);
        let mut packet = Vec::new();

        for chunk in message.as_bytes().chunks(self.block_capacity()) {
            let mut encoded = Vec::with_capacity(chunk.len() + 1);

            //A nonzero prefix preserves leading zero bytes in the payload.
            encoded.push(1);
            encoded.extend_from_slice(chunk);

            let message_number = BigUint::from_bytes_be(&encoded);
            let encrypted = self.encrypt_number(&message_number)?;
            let bytes = encrypted.to_bytes_be();

            //Fixed-width ciphertext blocks preserve packet boundaries.
            packet.resize(packet.len() + width - bytes.len(), 0);
            packet.extend_from_slice(&bytes);
        }

        Ok(packet)
    }
}

impl Receiver {
    pub fn decrypt_number(&self, ciphertext: &BigUint) -> Result<BigUint, String> {
        if ciphertext >= &self.n {
            return Err("The ciphertext integer must be smaller than n.".to_string());
        }

        Ok(ciphertext.modpow(&self.d, &self.n))
    }

    pub fn decrypt_text(&self, packet: &[u8]) -> Result<String, String> {
        let width = ciphertext_width(&self.n);

        if packet.len() % width != 0 {
            return Err("The packet contains an incomplete ciphertext block.".to_string());
        }

        let mut plaintext = Vec::new();

        for block in packet.chunks(width) {
            let ciphertext = BigUint::from_bytes_be(block);
            let recovered = self.decrypt_number(&ciphertext)?;
            let encoded = recovered.to_bytes_be();

            if encoded.first() != Some(&1)
                || encoded.len() < 2
                || encoded.len() > payload_capacity(&self.n) + 1
            {
                return Err("Invalid educational text encoding.".to_string());
            }

            plaintext.extend_from_slice(&encoded[1..]);
        }

        //Decode only after reassembling all bytes, including split UTF-8 characters.
        String::from_utf8(plaintext)
            .map_err(|_| "The recovered message is not valid UTF-8.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cryptography::verified::VerifiedPrime;

    const P: &str = "8226342136733210179679450416358439885889";
    const Q: &str = "4319206805826234237966605867347973035363";

    fn fixture() -> VerifiedBatch {
        VerifiedBatch {
            batch_id: "batch_test".to_string(),
            algorithm: "SHA-256 Counter".to_string(),
            primes: vec![
                VerifiedPrime {
                    index: 32,
                    number: BigUint::parse_bytes(P.as_bytes(), 10).unwrap(),
                },
                VerifiedPrime {
                    index: 148,
                    number: BigUint::parse_bytes(Q.as_bytes(), 10).unwrap(),
                },
            ],
            composites: 9998,
        }
    }

    #[test]
    fn recovers_numbers_including_values_not_coprime_to_the_modulus() {
        let batch = fixture();
        let (sender, receiver) = build_participants(&batch, 32, 148).unwrap();

        assert_eq!(
            sender.modulus(),
            &(&batch.primes[0].number * &batch.primes[1].number)
        );

        let messages = [
            BigUint::zero(),
            BigUint::one(),
            batch.primes[0].number.clone(),
            batch.primes[1].number.clone(),
            sender.modulus() - BigUint::one(),
        ];

        for message in messages {
            let encrypted = sender.encrypt_number(&message).unwrap();
            let recovered = receiver.decrypt_number(&encrypted).unwrap();

            assert_eq!(recovered, message);
        }
    }

    #[test]
    fn preserves_empty_text_zero_bytes_and_multiblock_unicode() {
        let (sender, receiver) = build_participants(&fixture(), 32, 148).unwrap();

        let messages = [
            String::new(),
            "\0\0Hello\0".to_string(),
            "  Hello, receiver! ñ 🔐  ".to_string(),
            format!("{}🔐{}", " ".repeat(31), "ñ".repeat(80)),
            P.to_string(),
        ];

        for message in messages {
            let packet = sender.encrypt_text(&message).unwrap();
            let recovered = receiver.decrypt_text(&packet).unwrap();

            assert_eq!(recovered, message);
        }
    }

    #[test]
    fn rejects_unknown_indices_and_repeated_prime_values() {
        let mut batch = fixture();

        assert!(build_participants(&batch, 32, 32).is_err());
        assert!(build_participants(&batch, 32, 9999).is_err());

        batch.primes[1].number = batch.primes[0].number.clone();

        assert!(build_participants(&batch, 32, 148).is_err());
    }

    #[test]
    fn rejects_out_of_range_numbers_and_incomplete_blocks() {
        let (sender, receiver) = build_participants(&fixture(), 32, 148).unwrap();

        assert!(sender.encrypt_number(sender.modulus()).is_err());
        assert!(receiver.decrypt_number(sender.modulus()).is_err());

        let mut packet = sender.encrypt_text("Test message").unwrap();
        packet.pop();

        assert!(receiver.decrypt_text(&packet).is_err());
    }
}