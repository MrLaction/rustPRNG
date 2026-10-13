mod generators;
mod number;
mod prime;
mod experiment;
mod cryptography;

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

fn select_generator() -> (Box<dyn PseudoRandomGenerator>, String) {
    loop {
        println!("\nPseudorandom Number Generator");
        println!("\nSelect an algorithm:");
        println!("1. SHA-256");
        println!("2. ChaCha20");
        println!("3. Blum Blum Shub");

        let option = read_line("> ");

        match option.as_str() {
            "1" => {
                println!("\nSHA-256 selected.\n");

                let seed = read_biguint("Seed: ");
                let salt = read_biguint("Salt: ");

                let parameters = format!(
                    "seed={seed}\nsalt={salt}"
                );

                return (
                    Box::new(Sha256Counter::new(seed, salt)),
                    parameters,
                );
            }

            "2" => {
                println!("\nChaCha20 selected.\n");

                let key_seed = read_biguint("Key seed: ");
                let nonce_seed = read_biguint("Nonce seed: ");

                let parameters = format!(
                    "key_seed={key_seed}\nnonce_seed={nonce_seed}"
                );

                return (
                    Box::new(ChaCha20Generator::new(key_seed, nonce_seed)),
                    parameters,
                );
            }

            "3" => {
                println!("\nBlum Blum Shub selected.");
                println!("p and q must be distinct probable primes.");
                println!("Both must satisfy value % 4 == 3.\n");

                let p = read_biguint("p: ");
                let q = read_biguint("q: ");
                let seed = read_biguint("Seed: ");

                let parameters = format!(
                    "p={p}\nq={q}\nseed={seed}"
                );

                match BlumBlumShub::new(p, q, seed) {
                    Ok(generator) => {
                        return (Box::new(generator), parameters);
                    }
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

fn run_number_experiments() {
    let (mut generator, _parameters) = select_generator();

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

fn run_coursework_batch() {
    let (mut generator, parameters) = select_generator();

    println!();
    println!("Coursework batch");
    println!("Algorithm: {}", generator.name());
    println!(
        "Generating and testing {} numbers...",
        cryptography::coursework::BATCH_SIZE
    );

    let saved = match cryptography::coursework::generate_batch(
        generator.as_mut(),
        &parameters,
    ) {
        Ok(saved) => saved,
        Err(error) => {
            eprintln!("Could not save the coursework batch: {error}");
            return;
        }
    };

    print_experiment_results(&saved.result);

    println!();
    println!("Batch directory: {}", saved.directory.display());
    println!("Saved files:");
    println!("  numbers.csv");
    println!("  metadata.txt");
    println!("  verification.gp");

    println!();
    println!("Primality verification is pending.");
    println!("Run verification.gp in PARI/GP.");
    println!("Save its CSV output as verified.csv in this batch directory.");
    println!("Keep the batch files to reuse these exact numbers later.");
}

fn run_verified_batch_import() {
    let directory = read_line("Batch directory: ");

    let batch = match cryptography::verified::load_batch(
        std::path::Path::new(&directory),
    ) {
        Ok(batch) => batch,
        Err(message) => {
            eprintln!("Batch import failed: {message}");
            return;
        }
    };

    println!();
    println!("External verification imported successfully.");
    println!("Batch: {}", batch.batch_id);
    println!("Algorithm: {}", batch.algorithm);
    println!(
        "Generated numbers: {}",
        cryptography::coursework::BATCH_SIZE
    );
    println!("Primes confirmed by PARI/GP: {}", batch.primes.len());
    println!("Composites: {}", batch.composites);
    println!("All candidate results match the original batch.");

    println!();
    println!("Verified primes:");

    for entry in &batch.primes {
        println!("{:>5}: {}", entry.index, entry.number);
    }

    println!();
    println!("Primality results were imported from PARI/GP.");
    println!("Rust checked file consistency without repeating the proof.");
}

fn main() {
    loop {
        println!();
        println!("Main menu");
        println!("1. Number generation and primality experiments");
        println!("2. Cryptography");
        println!("3. Coursework: 10000 numbers and prime verification");
        println!("4. Coursework: import PARI verification");
        println!("5. Coursework: RSA with verified 40-digit primes");
        println!("0. Exit");

        let option = read_line("> ");

        match option.as_str() {
            "1" => {
                run_number_experiments();
                return;
            }
            "2" => {
                cryptography::run();
                return;
            }
            "3" => {
                run_coursework_batch();
                return;
            }
            "4" => {
                run_verified_batch_import();
                return;
            }
            "5" => {
                cryptography::educational::run();
                return;
            }

            "0" => return,
            _ => println!("Invalid option."),
        }
    }
}