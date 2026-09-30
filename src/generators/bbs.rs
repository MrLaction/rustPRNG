use num_bigint::BigUint;
use num_traits::{One, Zero};

use super::PseudoRandomGenerator;
use crate::prime::is_probably_prime;

pub struct BlumBlumShub {
    modulus: BigUint,
    state: BigUint,
}

impl BlumBlumShub {
    pub fn new(
        p: BigUint,
        q: BigUint,
        seed: BigUint,
    ) -> Result<Self, &'static str> {
        let one = BigUint::one();
        let three = BigUint::from(3u32);
        let four = BigUint::from(4u32);

        if p == q {
            return Err("p and q must be different.");
        }

        if &p % &four != three {
            return Err("p must satisfy p % 4 == 3.");
        }

        if &q % &four != three {
            return Err("q must satisfy q % 4 == 3.");
        }

        if !is_probably_prime(&p) {
            return Err("p must pass the probable primality test.");
        }

        if !is_probably_prime(&q) {
            return Err("q must pass the probable primality test.");
        }

        let modulus = &p * &q;

        //Exclude trivial seeds and values outside the modulus.
        if seed <= one || seed >= modulus {
            return Err("Seed must satisfy 1 < seed < p * q.");
        }

        if gcd(seed.clone(), modulus.clone()) != one {
            return Err("Seed must be coprime with p * q.");
        }

        let state = (&seed * &seed) % &modulus;

        //Some coprime seeds still produce the fixed state 1.
        if state == one {
            return Err("Seed produces a fixed state. Choose another seed.");
        }

        Ok(Self { modulus, state })
    }

    fn next_bit(&mut self) -> u8 {
        self.state = (&self.state * &self.state) % &self.modulus;

        //Extract the least significant bit of the updated state.
        u8::from(self.state.bit(0))
    }
}

impl PseudoRandomGenerator for BlumBlumShub {
    fn fill_bytes(&mut self, output: &mut [u8]) {
        for byte in output.iter_mut() {
            *byte = 0;

            //Pack eight consecutive bits, most significant bit first.
            for _ in 0..8 {
                *byte = (*byte << 1) | self.next_bit();
            }
        }
    }

    fn name(&self) -> &'static str {
        "Blum Blum Shub"
    }
}

fn gcd(mut a: BigUint, mut b: BigUint) -> BigUint {
    //Euclidean algorithm.
    while !b.is_zero() {
        let remainder = &a % &b;
        a = b;
        b = remainder;
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::generate_40_digit_number;

    fn test_generator() -> BlumBlumShub {
        BlumBlumShub::new(
            BigUint::from(383u32),
            BigUint::from(503u32),
            BigUint::from(101u32),
        )
        .expect("Test parameters must be valid")
    }

    #[test]
    fn matches_reference_bytes() {
        let mut generator = test_generator();
        let mut output = [0u8; 16];

        generator.fill_bytes(&mut output);

        //Reference sequence for p = 383, q = 503, seed = 101.
        assert_eq!(
            output,
            [
                163, 221, 191, 93, 8, 0, 38, 196,
                183, 240, 26, 88, 250, 219, 11, 246,
            ]
        );
    }

    #[test]
    fn rejects_invalid_parameters() {
        let invalid_cases: [(u32, u32, u32); 10] = [
            (383, 383, 101),
            (13, 503, 101),
            (383, 13, 101),
            (15, 503, 101),
            (383, 15, 101),
            (383, 503, 0),
            (383, 503, 1),
            (383, 503, 383),
            (383, 503, 383 * 503),
            (383, 503, 383 * 503 - 1),
        ];

        for (p, q, seed) in invalid_cases {
            let result = BlumBlumShub::new(
                BigUint::from(p),
                BigUint::from(q),
                BigUint::from(seed),
            );

            assert!(
                result.is_err(),
                "Expected rejection for p={p}, q={q}, seed={seed}"
            );
        }
    }

    #[test]
    fn output_is_independent_of_buffer_contents_and_chunk_sizes() {
        let mut whole_generator = test_generator();
        let mut split_generator = test_generator();

        let mut whole = [0u8; 128];
        let mut split = [0xffu8; 128];

        whole_generator.fill_bytes(&mut whole);

        for chunk in split.chunks_mut(17) {
            split_generator.fill_bytes(chunk);
        }

        assert_eq!(whole, split);
    }

    #[test]
    fn generated_numbers_have_40_digits_and_are_reproducible() {
        let mut generator_a = test_generator();
        let mut generator_b = test_generator();

        for _ in 0..100 {
            let a = generate_40_digit_number(&mut generator_a);
            let b = generate_40_digit_number(&mut generator_b);

            assert_eq!(a.to_string().len(), 40);
            assert_eq!(a, b);
        }
    }
}