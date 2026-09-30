mod generators;
mod number;
mod prime;

use std::io::{self, Write};

use generators::PseudoRandomGenerator;
use generators::sha256::Sha256Counter;
use generators::chacha20::ChaCha20Generator;
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

            "3" => println!("Blum Blum Shub is not implemented yet."),
            _ => println!("Invalid option."),
        }
    }
}

fn main() {
    let mut generator = select_generator();

    println!();
    println!("Algorithm: {}", generator.name());

    let count = read_usize("How many numbers do you want to generate?: ");

    println!();

    for i in 1..=count {
        let number = generate_40_digit_number(generator.as_mut());
        println!("{i:>5}: {number}");
    }
}
