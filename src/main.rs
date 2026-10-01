mod generators;
mod number;
mod prime;
mod experiment;

use std::io::{self, Write};

use generators::PseudoRandomGenerator;
use generators::sha256::Sha256Counter;
use generators::chacha20::ChaCha20Generator;
use generators::bbs::BlumBlumShub;
use experiment::{ExperimentResult, run_experiment};
use num_bigint::BigUint;
use number::generate_40_digit_number;

fn read_line(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}

fn read_biguint(prompt: &str) -> BigUint {
    loop {
        let input = read_line(prompt);

        match BigUint::parse_bytes(input.as_bytes(), 10) {
            Some(value) => return value,
            None => println!("Invalid input. Please enter a non-negative integer."),
        }
    }
}

fn read_usize(prompt: &str) -> usize {
    loop {
        let input = read_line(prompt);

        match input.parse::<usize>() {
            Ok(value) if value > 0 => return value,
            _ => println!("Please enter a positive integer."),
        }
    }
}

fn select_generator() -> Box<dyn PseudoRandomGenerator> {
    loop {
        println!("\nPseudorandom Number Generator");
        println!("\nSelect an algorithm:");
        println!("1. SHA-256");
        println!("2. ChaCha20");
        println!("3. Blum Blum Shub");

        let option = read_line("> ");

        match option.as_str() {
            "1" => {
                println!();
                println!("SHA-256 selected.");
                println!();

                let seed = read_biguint("Seed: ");
                let salt = read_biguint("Salt: ");

                return Box::new(Sha256Counter::new(seed, salt));
            }

            "2" => {
                println!();
                println!("ChaCha20 selected.");
                println!();

                let key_seed = read_biguint("Key seed: ");
                let nonce_seed = read_biguint("Nonce seed: ");

                return Box::new(ChaCha20Generator::new(key_seed, nonce_seed));
            }

            "3" => {
                println!();
                println!("Blum Blum Shub selected.");
                println!("p and q must be distinct probable primes.");
                println!("Both must satisfy value % 4 == 3.");
                println!();

                let p = read_biguint("p: ");
                let q = read_biguint("q: ");
                let seed = read_biguint("Seed: ");

                match BlumBlumShub::new(p, q, seed) {
                    Ok(generator) => return Box::new(generator),
                    Err(message) => {
                        println!("Invalid BBS parameters: {message}");
                    }
                }
            }
            _ => println!("Invalid option."),
        }
    }
}

fn select_mode() -> u8 {
    loop {
        println!();
        println!("Select a mode:");
        println!("1. Generate numbers");
        println!("2. Generate numbers and count probable primes");

        let option = read_line("> ");

        match option.as_str() {
            "1" => return 1,
            "2" => return 2,
            _ => println!("Invalid option."),
        }
    }
}

fn print_experiment_results(result: &ExperimentResult) {
    println!();
    println!("Generated numbers:");
    println!();

    //Print the stored results without generating or testing again.
    for (index, entry) in result.numbers.iter().enumerate() {
        let label = if entry.probably_prime {
            "probably prime"
        } else {
            "not prime"
        };

        println!("{:>5}: {} | {}", index + 1, entry.number, label);
    }

    let generated = result.numbers.len();
    let composites = generated - result.probable_primes;

    let percentage = if generated == 0 {
        0.0
    } else {
        result.probable_primes as f64 / generated as f64 * 100.0
    };

    println!();
    println!("Experiment summary");
    println!("Generated numbers: {generated}");
    println!("Probable primes: {}", result.probable_primes);
    println!("Composites: {composites}");
    println!("Probable prime percentage: {percentage:.4}%");
    println!(
        "Generation and testing time: {:.3} s",
        result.elapsed.as_secs_f64()
    );
}

fn main() {
    let mut generator = select_generator();

    println!();
    println!("Algorithm: {}", generator.name());

    let mode = select_mode();
    let count = read_usize("How many numbers do you want to generate? ");

    match mode {
        1 => {
            println!();

            for index in 1..=count {
                let number = generate_40_digit_number(generator.as_mut());
                println!("{index:>5}: {number}");
            }
        }

        2 => {
            println!();
            println!("Generating and testing {count} numbers...");

            let result = run_experiment(generator.as_mut(), count);

            print_experiment_results(&result);
        }

        _ => unreachable!("Mode was validated by select_mode"),
    }
}