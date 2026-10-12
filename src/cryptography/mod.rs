mod channel;
mod keys;
mod primes;
mod receiver;
mod transmitter;

use std::io::{self, Write};
use std::time::Instant;

use rsa::traits::PublicKeyParts;

use channel::Channel;
use receiver::Receiver;
use transmitter::Transmitter;

pub fn run() {
    if let Err(message) = run_simulation() {
        eprintln!("Cryptography error: {message}");
    }
}

fn read_message(maximum_bytes: usize) -> Result<String, String> {
    loop {
        print!("\nEnter a message: ");
        io::stdout()
            .flush()
            .map_err(|error| format!("Could not flush output: {error}"))?;

        let mut input = String::new();

        let bytes_read = io::stdin()
            .read_line(&mut input)
            .map_err(|error| format!("Could not read input: {error}"))?;

        if bytes_read == 0 {
            return Err("Input was closed.".to_string());
        }

        //Remove the line ending while preserving message whitespace.
        if input.ends_with('\n') {
            input.pop();

            if input.ends_with('\r') {
                input.pop();
            }
        }

        if input.len() <= maximum_bytes {
            return Ok(input);
        }

        println!(
            "Message too long: {} bytes provided, {maximum_bytes} allowed.",
            input.len()
        );
    }
}

fn run_simulation() -> Result<(), String> {
    println!();
    println!("Cryptography");
    println!("Stage 3: transmitter, channel, and receiver");
    println!();
    println!("Generator: ChaCha20 with system-generated seeds");
    println!("Target: two distinct 1024-bit probable primes");
    println!("Primality test: Miller-Rabin with 32 rounds");
    println!("THESE RESULTS ARE NOT PRIMALITY PROOFS");
    println!();
    println!("Generating receiver factors...");

    let prime_start = Instant::now();
    let pair = primes::generate_prime_pair()?;
    let prime_elapsed = prime_start.elapsed();

    println!();
    println!("Prime generation summary");
    println!("Candidates generated: {}", pair.candidates_tested);
    println!("p bit length: {}", pair.p.bits());
    println!("q bit length: {}", pair.q.bits());
    println!(
        "Generation and testing time: {:.3} s",
        prime_elapsed.as_secs_f64()
    );

    let key_start = Instant::now();
    let (private_key, public_key) = keys::build_key_pair(&pair)?;
    let key_elapsed = key_start.elapsed();

    //Key construction has finished using the generated factors.
    drop(pair);

    println!();
    println!("Receiver public key");
    println!("n = {}", public_key.n());
    println!("e = {}", public_key.e());
    println!("Modulus bit length: {}", public_key.n().bits());
    println!(
        "Key construction time: {:.3} s",
        key_elapsed.as_secs_f64()
    );

    let receiver = Receiver::new(private_key);
    let transmitter = Transmitter::new(public_key);
    let mut channel = Channel::new();

    println!();
    println!("Encryption scheme: RSA-OAEP with SHA-256");
    println!("The transmitter receives only the public key.");
    println!(
        "Maximum message length: {} UTF-8 bytes",
        transmitter.max_message_bytes()
    );

    let original = read_message(transmitter.max_message_bytes())?;

    let encryption_start = Instant::now();
    let ciphertext = transmitter.encrypt(&original)?;
    let encryption_elapsed = encryption_start.elapsed();

    println!();
    println!("Transmitter");
    println!("Original message: {original}");
    println!("Original size: {} bytes", original.len());

    println!();
    println!("Channel");
    println!("Ciphertext size: {} bytes", ciphertext.len());
    println!("Ciphertext in hexadecimal:");

    for byte in &ciphertext {
        print!("{byte:02x}");
    }
    println!();

    channel.send(ciphertext);

    let packet = channel
        .receive()
        .ok_or_else(|| "The channel contains no message.".to_string())?;

    let decryption_start = Instant::now();
    let recovered = receiver.decrypt(&packet)?;
    let decryption_elapsed = decryption_start.elapsed();

    println!();
    println!("Receiver");
    println!("Recovered message: {recovered}");

    if recovered != original {
        return Err("The recovered message differs from the original.".to_string());
    }

    println!();
    println!("Transmission summary");
    println!("Exact message match: yes");
    println!(
        "Encryption time: {:.3} ms",
        encryption_elapsed.as_secs_f64() * 1000.0
    );
    println!(
        "Decryption time: {:.3} ms",
        decryption_elapsed.as_secs_f64() * 1000.0
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transmission_preserves_messages_and_rejects_truncated_packets() {
        let pair = primes::generate_prime_pair()
            .expect("Prime generation failed");

        let (private_key, public_key) = keys::build_key_pair(&pair)
            .expect("Key construction failed");

        let receiver = Receiver::new(private_key);
        let transmitter = Transmitter::new(public_key);
        let mut channel = Channel::new();

        let messages = [
            String::new(),
            "  Hello, receiver! ñ 🔐  ".to_string(),
            "a".repeat(190),
        ];

        //Send every message before receiving to exercise channel ordering.
        for message in &messages {
            let ciphertext = transmitter
                .encrypt(message)
                .expect("Encryption failed");

            channel.send(ciphertext);
        }

        for expected in &messages {
            let packet = channel.receive().expect("Missing packet");
            let recovered = receiver
                .decrypt(&packet)
                .expect("Decryption failed");

            assert_eq!(&recovered, expected);
        }

        assert!(channel.receive().is_none());

        //Ninety-six copies occupy 192 UTF-8 bytes.
        assert!(transmitter.encrypt(&"ñ".repeat(96)).is_err());

        let mut truncated = transmitter
            .encrypt("Packet integrity test")
            .expect("Encryption failed");

        truncated.pop();

        assert!(receiver.decrypt(&truncated).is_err());
    }
}