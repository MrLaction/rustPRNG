use num_bigint::BigUint;
use num_traits::{One, Zero};
use rand::RngCore;

const MILLER_RABIN_ROUNDS: usize = 32;

const SMALL_PRIMES: [u32; 12] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37,
];

pub fn is_probably_prime(number: &BigUint) -> bool {
    let mut rng = rand::thread_rng();
    is_probably_prime_with_rng(number, &mut rng)
}

fn is_probably_prime_with_rng(
    number: &BigUint,
    rng: &mut impl RngCore,
) -> bool {
    let one = BigUint::one();
    let two = BigUint::from(2u32);

    //Zero and one are not prime.
    if number < &two {
        return false;
    }

    //Handle small primes and reject their multiples.
    for value in SMALL_PRIMES {
        let prime = BigUint::from(value);

        if number == &prime {
            return true;
        }

        if (number % &prime).is_zero() {
            return false;
        }
    }

    //At this point, number is odd and greater than 37.
    //Decompose number - 1 as d * 2^s, with d odd.
    let number_minus_one = number - &one;
    let mut d = number_minus_one.clone();
    let mut s = 0u64;

    while !d.bit(0) {
        d >>= 1usize;
        s += 1;
    }

    //Sampling below number - 3 and adding 2 gives [2, number - 2].
    let base_range = number - BigUint::from(3u32);

    for _ in 0..MILLER_RABIN_ROUNDS {
        let base = random_below(&base_range, rng) + &two;

        if !passes_miller_rabin_round(
            number,
            &number_minus_one,
            &d,
            s,
            &base,
        ) {
            return false;
        }
    }

    true
}

fn passes_miller_rabin_round(
    number: &BigUint,
    number_minus_one: &BigUint,
    d: &BigUint,
    s: u64,
    base: &BigUint,
) -> bool {
    let one = BigUint::one();
    let mut x = base.modpow(d, number);

    if x == one || &x == number_minus_one {
        return true;
    }

    for _ in 1..s {
        x = (&x * &x) % number;

        if &x == number_minus_one {
            return true;
        }

        //Reaching one before minus one reveals a composite.
        if x == one {
            return false;
        }
    }

    false
}

fn random_below(
    upper_bound: &BigUint,
    rng: &mut impl RngCore,
) -> BigUint {
    assert!(
        !upper_bound.is_zero(),
        "The upper bound must be positive"
    );

    let bits = upper_bound.bits();
    let byte_count = usize::try_from(bits.div_ceil(8))
        .expect("The upper bound is too large");

    let unused_bits = ((8 - bits % 8) % 8) as u32;
    let mut bytes = vec![0u8; byte_count];

    loop {
        rng.fill_bytes(&mut bytes);

        //Discard unused high bits before applying rejection sampling.
        bytes[0] &= u8::MAX >> unused_bits;

        let candidate = BigUint::from_bytes_be(&bytes);

        if &candidate < upper_bound {
            return candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn is_prime_by_trial_division(number: u32) -> bool {
        if number < 2 {
            return false;
        }

        let mut divisor = 2;

        while divisor <= number / divisor {
            if number % divisor == 0 {
                return false;
            }

            divisor += 1;
        }

        true
    }

    #[test]
    fn matches_exact_results_for_small_numbers() {
        let mut rng = StdRng::seed_from_u64(42);

        for value in 0..1000u32 {
            let number = BigUint::from(value);

            assert_eq!(
                is_probably_prime_with_rng(&number, &mut rng),
                is_prime_by_trial_division(value),
                "Unexpected result for {value}"
            );
        }
    }

    #[test]
    fn handles_large_primes_and_composites() {
        let mut rng = StdRng::seed_from_u64(12345);

        //2^127 - 1 is a known Mersenne prime.
        let prime = (BigUint::one() << 127usize) - BigUint::one();

        assert!(is_probably_prime_with_rng(&prime, &mut rng));

        //This square exceeds u128 and has no small prime factor.
        let composite = &prime * &prime;

        assert!(!is_probably_prime_with_rng(&composite, &mut rng));
    }

    #[test]
    fn another_base_detects_a_base_two_pseudoprime() {
        //2047 = 23 * 89 passes base 2 but fails base 3.
        let number = BigUint::from(2047u32);
        let number_minus_one = BigUint::from(2046u32);
        let d = BigUint::from(1023u32);
        let s = 1;

        assert!(passes_miller_rabin_round(
            &number,
            &number_minus_one,
            &d,
            s,
            &BigUint::from(2u32),
        ));

        assert!(!passes_miller_rabin_round(
            &number,
            &number_minus_one,
            &d,
            s,
            &BigUint::from(3u32),
        ));
    }
}