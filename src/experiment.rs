use std::time::{Duration, Instant};

use num_bigint::BigUint;

use crate::generators::PseudoRandomGenerator;
use crate::number::generate_40_digit_number;
use crate::prime::is_probably_prime;

pub struct NumberResult {
    pub number: BigUint,
    pub probably_prime: bool,
}

pub struct ExperimentResult {
    pub numbers: Vec<NumberResult>,
    pub probable_primes: usize,
    pub elapsed: Duration,
}

pub fn run_experiment<G>(
    generator: &mut G,
    count: usize,
) -> ExperimentResult
where
    G: PseudoRandomGenerator + ?Sized,
{
    let start = Instant::now();

    let mut numbers = Vec::with_capacity(count);
    let mut probable_primes = 0;

    for _ in 0..count {
        let number = generate_40_digit_number(generator);
        let probably_prime = is_probably_prime(&number);

        if probably_prime {
            probable_primes += 1;
        }

        //Store each number and its result in generation order.
        numbers.push(NumberResult {
            number,
            probably_prime,
        });
    }

    //Stop timing before formatting or printing the results.
    let elapsed = start.elapsed();

    ExperimentResult {
        numbers,
        probable_primes,
        elapsed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedGenerator {
        offsets: [u8; 3],
        position: usize,
    }

    impl PseudoRandomGenerator for FixedGenerator {
        fn fill_bytes(&mut self, output: &mut [u8]) {
            assert_eq!(output.len(), 17);

            output.fill(0);
            output[16] = self.offsets[self.position];
            self.position += 1;
        }

        fn name(&self) -> &'static str {
            "Fixed test generator"
        }
    }

    #[test]
    fn preserves_generation_order_and_classifies_known_composites() {
        let mut generator = FixedGenerator {
            offsets: [2, 0, 1],
            position: 0,
        };

        let result = run_experiment(&mut generator, 3);
        let minimum = BigUint::from(10u32).pow(39);

        assert_eq!(result.numbers.len(), 3);
        assert_eq!(result.probable_primes, 0);
        assert_eq!(generator.position, 3);

        //10^39 and 10^39 + 2 are even.
        //10^39 + 1 is divisible by 11.
        for (entry, offset) in result.numbers.iter().zip([2u32, 0, 1]) {
            assert_eq!(
                entry.number,
                &minimum + BigUint::from(offset)
            );
            assert!(!entry.probably_prime);
        }
    }
}