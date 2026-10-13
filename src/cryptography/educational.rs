use std::io::{self, Write};
use std::path::Path;

use num_bigint::BigUint;

use super::channel::Channel;
use super::{textbook, verified};

pub fn run() {
    if let Err(message) = run_simulation() {
        eprintln!("Educational RSA error: {message}");
    }
}

fn read_input(prompt: &str) -> Result<String, String> {
    print!("{prompt}");

    io::stdout()
        .flush()
        .map_err(|error| error.to_string())?;

    let mut input = String::new();

    let count = io::stdin()
        .read_line(&mut input)
        .map_err(|error| error.to_string())?;

    if count == 0 {
        return Err("Input was closed.".to_string());
    }

    if input.ends_with('\n') {
        input.pop();

        if input.ends_with('\r') {
            input.pop();
        }
    }

    Ok(input)
}

fn read_index(prompt: &str) -> Result<usize, String> {
    read_input(prompt)?
        .trim()
        .parse::<usize>()
        .map_err(|_| "Enter a valid batch index.".to_string())
}

fn transmit(packet: Vec<u8>) -> Result<Vec<u8>, String> {
    println!();
    println!("Channel");
    println!("Packet size: {} bytes", packet.len());
    println!("Ciphertext in hexadecimal:");

    for byte in &packet {
        print!("{byte:02x}");
    }
    println!();

    let mut channel = Channel::new();
    channel.send(packet);

    channel
        .receive()
        .ok_or_else(|| "The channel contains no packet.".to_string())
}

fn run_simulation() -> Result<(), String> {
    println!();
    println!("Educational RSA using verified 40-digit primes");
    println!("This demonstration uses textbook RSA without OAEP.");
    println!("It is not intended to protect real information.");
    println!();

    let directory = read_input("Batch directory: ")?;
    let batch = verified::load_batch(Path::new(directory.trim()))?;

    println!();
    println!("Batch: {}", batch.batch_id);
    println!("Generator: {}", batch.algorithm);
    println!("Verified prime entries: {}", batch.primes.len());
    println!();
    println!("Available primes, identified by original batch index:");

    for entry in &batch.primes {
        println!("{:>5}: {}", entry.index, entry.number);
    }

    println!();
    let p_index = read_index("Original index for p: ")?;
    let q_index = read_index("Original index for q: ")?;

    let (sender, receiver) = textbook::build_participants(
        &batch,
        p_index,
        q_index,
    )?;

    println!();
    println!("RSA key constructed from the selected batch values.");
    println!("p source index: {p_index}");
    println!("q source index: {q_index}");
    println!("n = {}", sender.modulus());
    println!("e = {}", sender.exponent());
    println!("Modulus bit length: {}", sender.modulus().bits());
    println!("Text payload per block: {} bytes", sender.block_capacity());

    println!();
    println!("Select a message type:");
    println!("1. UTF-8 text");
    println!("2. Non-negative integer smaller than n");

    let mode = read_input("> ")?;

    match mode.trim() {
        "1" => {
            let original = read_input("Message: ")?;

            println!();
            println!("Transmitter");
            println!("Original message: {original}");

            let packet = sender.encrypt_text(&original)?;
            let received = transmit(packet)?;
            let recovered = receiver.decrypt_text(&received)?;

            println!();
            println!("Receiver");
            println!("Recovered message: {recovered}");

            if recovered != original {
                return Err("The recovered text differs from the original.".to_string());
            }
        }

        "2" => {
            let input = read_input("Message integer: ")?;

            let original = BigUint::parse_bytes(
                input.trim().as_bytes(),
                10,
            )
            .ok_or_else(|| "Enter a non-negative decimal integer.".to_string())?;

            println!();
            println!("Transmitter");
            println!("Original integer: {original}");

            let ciphertext = sender.encrypt_number(&original)?;
            println!("Ciphertext integer: {ciphertext}");

            let received = transmit(ciphertext.to_bytes_be())?;
            let received_number = BigUint::from_bytes_be(&received);
            let recovered = receiver.decrypt_number(&received_number)?;

            println!();
            println!("Receiver");
            println!("Recovered integer: {recovered}");

            if recovered != original {
                return Err("The recovered integer differs from the original.".to_string());
            }
        }

        _ => return Err("Invalid message type.".to_string()),
    }

    println!();
    println!("Exact message match: yes");
    println!("The RSA factors came from the verified batch.");

    Ok(())
}