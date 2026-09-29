use num_bigint::BigUint;
use num_traits::FromPrimitive;

use crate::generators::PseudoRandomGenerator;

const RANDOM_BYTES: usize = 17;

fn min_40_digits() -> BigUint {
    BigUint::from_u8(10).unwrap().pow(39)
}

fn range_40_digits() -> BigUint {
    BigUint::from_u8(9).unwrap() * min_40_digits()
}

pub fn generate_40_digit_number<G>(generator: &mut G) -> BigUint
where
    G: PseudoRandomGenerator + ?Sized,
{
    let min = min_40_digits();
    let range = range_40_digits();

    loop {
        //17 bytes = 136 bits. Keep only 133 bits because
        //10^40 < 2^133, reducing unnecessary rejections.
        let mut bytes = [0u8; RANDOM_BYTES];
        generator.fill_bytes(&mut bytes);

        //Keep only the five least significant bits of the first byte:
        //136 - 3 = 133 bits.
        bytes[0] &= 0b0001_1111;

        let candidate = BigUint::from_bytes_be(&bytes);

        //Rejection sampling over [0, 9*10^39) avoids modulo bias.
        if candidate < range {
            return &min + candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::sha256::Sha256Counter;

    #[test]
    fn generated_numbers_always_have_40_digits() {
        let seed = BigUint::from(12345u64);
        let salt = BigUint::from(67890u64);
        let mut generator = Sha256Counter::new(seed, salt);

        for _ in 0..1000 {
            let number = generate_40_digit_number(&mut generator);
            assert_eq!(number.to_string().len(), 40);
        }
    }

    #[test]
    fn same_parameters_generate_same_sequence() {
        let mut generator_a = Sha256Counter::new(
            BigUint::from(12345u64),
            BigUint::from(67890u64),
        );

        let mut generator_b = Sha256Counter::new(
            BigUint::from(12345u64),
            BigUint::from(67890u64),
        );

        for _ in 0..100 {
            let a = generate_40_digit_number(&mut generator_a);
            let b = generate_40_digit_number(&mut generator_b);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn changing_salt_changes_sequence() {
        let mut generator_a = Sha256Counter::new(
            BigUint::from(12345u64),
            BigUint::from(67890u64),
        );

        let mut generator_b = Sha256Counter::new(
            BigUint::from(12345u64),
            BigUint::from(67891u64),
        );

        let a = generate_40_digit_number(&mut generator_a);
        let b = generate_40_digit_number(&mut generator_b);

        assert_ne!(a, b);
    }
}
