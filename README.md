# rustPRNG

Generate 40-digit pseudorandom numbers with three algorithms and evaluate their primality in Rust.


The current version reports probable primes, not proven primes. It uses our implementation of Miller-Rabin with 32 randomized rounds. No definitive primality-proving stage is included.

## Getting started

Install Rust and Cargo, then run:

```bash
git clone https://github.com/MrLaction/rustPRNG.git
cd rustPRNG
cargo test
cargo run --release
```

Use a recent stable Rust toolchain compatible with edition 2024. Cargo downloads the required dependencies during the first build. No Python or PARI/GP installation is needed for the Rust application.

Use `cargo run` during development and `cargo run --release` for performance measurements.

## Using the program

The program asks for the algorithm first, followed by its parameters, the operating mode, and the number of values to generate.

| Option | Algorithm | Parameters |
| --- | --- | --- |
| `1` | SHA-256 Counter | `seed`, `salt` |
| `2` | ChaCha20 | `key_seed`, `nonce_seed` |
| `3` | Blum Blum Shub | `p`, `q`, `seed` |

The two operating modes are:

- **Generate numbers:** print the requested numbers without testing primality.
- **Generate numbers and count probable primes:** generate and classify the entire batch, then print every result and a summary.

Example input for the SHA-256 experiment:

```text
Algorithm: 1
Seed: 109281
Salt: 102939
Mode: 2
Count: 100000
```

Enter each value when prompted. The labels above describe the input order.

In the second mode, each output line ends with `probably prime` or `not prime`. Results are stored in generation order and printed only after processing finishes. The program does not regenerate or retest the numbers when printing them.

## Algorithms

### SHA-256 Counter

Each block hashes a domain identifier, length-prefixed seed and salt encodings, and an incrementing counter. The project defines this construction; the `sha2` crate provides SHA-256.

The counter starts at zero. Unused bytes from a hash block are discarded by the current implementation. Reproducing the sequence therefore requires the same parameters and the same pattern of byte requests, as provided by the shared number sampler.

### ChaCha20

SHA-256 derives a 32-byte key and a 12-byte nonce from the two numeric input parameters, using separate domain identifiers. The `chacha20` crate produces the keystream and retains its position between calls.

### Blum Blum Shub

BBS is implemented directly in the project. It initializes a state and repeatedly squares it modulo the product of two primes:

```text
M = p * q
x_0 = seed^2 mod M
x_(i+1) = x_i^2 mod M
```

Each step emits the least significant bit of the updated state. Eight bits form one output byte.

The program requires distinct probable primes `p` and `q`, both congruent to `3` modulo `4`. It also checks `1 < seed < p * q`, coprimality of the seed and modulus, and rejection of the fixed initial state `1`.

Small parameters can produce short cycles. Having 40 output digits alone does not guarantee a long period or good statistical properties.

## Number conversion and primality

All three generators feed the same sampler:

1. Request 17 bytes and mask off the top three bits, leaving 133 bits.
2. Interpret the bytes as a non-negative integer `r`.
3. Accept only if `r < 9 * 10^39`; otherwise retry.
4. Return `10^39 + r`.

Every accepted result is in `[10^39, 10^40)` and has exactly 40 decimal digits. Rejection sampling avoids modulo bias when supplied with uniform bits. The program stops with an error after 1,024 consecutive rejected candidates.

The primality module handles small values and small-prime divisibility before applying Miller-Rabin. It decomposes `n - 1`, samples independent test bases, and evaluates modular powers and repeated squares.

| Classification | Meaning |
| --- | --- |
| `not prime` | The test established that the number is not prime. |
| `probably prime` | The number passed all configured checks, but no primality proof was produced. |

The decision logic is implemented in `src/prime.rs`. `num-bigint` supplies large-integer arithmetic and modular exponentiation; `rand` supplies the bytes used to choose test bases.

Generated sequences are reproducible for fixed generator settings. Miller-Rabin bases are sampled separately on each run, so execution paths and timing may vary. A false positive remains theoretically possible.

## Experimental results

The following results were obtained on the development machine using release builds, sequential execution, and 100,000 numbers per algorithm.

| Algorithm | Probable primes | Composites | Probable-prime percentage | Time |
| --- | ---: | ---: | ---: | ---: |
| SHA-256 Counter | 1,062 | 98,938 | 1.0620% | 0.729 s |
| ChaCha20 | 1,080 | 98,920 | 1.0800% | 0.760 s |
| Blum Blum Shub | 1,103 | 98,897 | 1.1030% | 2.554 s |

The three runs generated **300,000 numbers**, of which **3,245 were classified as probable primes**.

Parameters used:

| Algorithm | Configuration |
| --- | --- |
| SHA-256 Counter | `seed = 109281`, `salt = 102939` |
| ChaCha20 | `key_seed = 109281`, `nonce_seed = 102939` |
| Blum Blum Shub | `p = 9223372036854876599`, `q = 9223372036854989879`, `seed = 101` |

The reported development environment was Debian, an AMD Ryzen 7 8745HX, and approximately 31 GiB of RAM. The application does not use the GPU or parallel processing.

The timer includes batch allocation, generation, primality testing, and result storage. It excludes compilation, user input, generator initialization, BBS parameter validation, and printing.

These measurements are single runs, not a repeated benchmark study. More probable primes do not imply a better generator, and similar counts do not prove uniformity or cryptographic security.

## Testing and project structure

```bash
cargo test
```

The completed version passed **14 tests** on the development machine. Coverage includes 40-digit output, reproducibility, parameter changes, BBS validation and reference bytes, buffer handling, primality cases, and preservation of experiment order.

| File | Purpose |
| --- | --- |
| `src/main.rs` | Interactive menus, input, and printed results. |
| `src/experiment.rs` | Batch execution, classifications, counts, and timing. |
| `src/number.rs` | Shared conversion to 40-digit numbers. |
| `src/prime.rs` | Miller-Rabin implementation and supporting functions. |
| `src/generators/mod.rs` | Common generator interface. |
| `src/generators/sha256.rs` | SHA-256 Counter generator. |
| `src/generators/chacha20.rs` | ChaCha20 generator. |
| `src/generators/bbs.rs` | Blum Blum Shub generator. |

## Scope

Users control the technique, initial parameters, operating mode, and batch size. Input validation constrains configurations; reproducible output and summaries make the system's behavior observable.

This version completes the educational generation and probable-prime-counting experiment. Definitive primality proofs, parallel execution, and persistent caches are outside its scope. Memory usage grows with the batch size because classified results are retained until printing.
