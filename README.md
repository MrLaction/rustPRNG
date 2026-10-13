# rustPRNG

Generate pseudorandom numbers, verify large primes, and use those primes to simulate encrypted communication in Rust.

The project includes:

- Three algorithms for generating 40-digit pseudorandom numbers.
- Batch generation and probable-prime classification with Miller-Rabin.
- Export of 10,000-number batches for external verification with PARI/GP.
- Import and consistency checks for the verification results.
- Educational RSA using two verified primes selected from the original batch.
- An additional RSA-OAEP simulation using newly generated 1024-bit factors.

The coursework workflow connects the original experiment directly to encryption: the RSA factors are selected from the numbers generated in the saved batch.

## Getting started

Install Rust and Cargo, then run:

```bash
git clone https://github.com/MrLaction/rustPRNG.git
cd rustPRNG
cargo test --release
cargo run --release
```

Use a recent stable Rust toolchain compatible with edition 2024. Cargo downloads the dependencies during the first build.

Release mode is recommended for experiments and tests involving large integers. Use `cargo run` for development builds.

The Rust application does not require Python or a local PARI/GP installation. The coursework verification step can be performed using PARI/GP in a browser.

## Using the program

The main menu provides five options:

```text
Main menu
1. Number generation and primality experiments
2. Cryptography
3. Coursework: 10000 numbers and prime verification
4. Coursework: import PARI verification
5. Coursework: RSA with verified 40-digit primes
0. Exit
```

| Option | Purpose |
| --- | --- |
| `1` | Run the original number-generation experiments with a configurable batch size. |
| `2` | Generate new 1024-bit factors and demonstrate RSA-OAEP communication. |
| `3` | Generate and export exactly 10,000 numbers of 40 decimal digits. |
| `4` | Import external primality results and check their correspondence with the saved batch. |
| `5` | Encrypt text or integers using two verified primes from that batch. |

### Original number experiments

Select `1`, choose a generator, and enter its parameters.

| Generator option | Algorithm | Parameters |
| --- | --- | --- |
| `1` | SHA-256 Counter | `seed`, `salt` |
| `2` | ChaCha20 | `key_seed`, `nonce_seed` |
| `3` | Blum Blum Shub | `p`, `q`, `seed` |

Two operating modes are available:

- **Generate numbers:** print the requested values without testing primality.
- **Generate numbers and count probable primes:** classify the complete batch, then print each value and a summary.

Example input sequence:

```text
Main menu: 1
Algorithm: 1
Seed: 109281
Salt: 102939
Mode: 2
Count: 100000
```

The labels describe the prompt order. Enter only the corresponding values.

Classified numbers are stored in generation order and printed after processing. Printing does not regenerate or retest them.

### Coursework workflow

The coursework workflow uses options `3`, `4`, and `5`.

#### Generate the batch

Select `3`, choose one of the three generators, and provide its parameters.

The program generates exactly 10,000 numbers and creates a directory such as:

```text
results/batch_1791902784183277849_6753/
```

Each execution receives a separate directory.

| File | Contents |
| --- | --- |
| `numbers.csv` | Every generated number, its original index, and its Miller-Rabin classification. |
| `metadata.txt` | Batch identifier, algorithm, parameters, counts, and generation time. |
| `verification.gp` | PARI/GP instructions for checking every probable-prime candidate. |

The batch identifier links the files. Original indices remain unchanged when candidates are exported.

#### Verify the candidates in PARI/GP

Open `verification.gp` and execute its complete contents in PARI/GP.

The script calls `isprime` for each candidate:

- `1` means PARI established that the candidate is prime.
- `0` means PARI established that it is not prime.

Save the resulting CSV header and data rows as `verified.csv` inside the same batch directory. Do not include browser interface text, GP prompts, or the original `print(...)` instructions.

The output begins with:

```text
batch_id,index,number,is_prime
```

This is an excerpt from the recorded verification:

```text
batch_id,index,number,is_prime
batch_1791902784183277849_6753,32,8226342136733210179679450416358439885889,1
batch_1791902784183277849_6753,148,4319206805826234237966605867347973035363,1
```

The complete file must contain a result for every candidate, not only the two primes later selected for RSA.

PARI/GP performs the primality decision externally. Rust imports the reported results; it does not execute PARI or validate a mathematical primality certificate.

#### Import the verification

Select `4` and provide the batch directory.

The importer checks:

- The metadata format and required counts.
- Exactly 10,000 number rows.
- Consecutive original indices.
- Exactly 40 decimal digits per number.
- Agreement between metadata and Miller-Rabin classifications.
- Matching batch identifiers and exact candidate values.
- Verification flags restricted to `0` or `1`.
- No missing, duplicate, or unexpected candidate results.

Candidates rejected by PARI are excluded from the verified-prime list.

The original files remain unchanged. In particular, `verification_status=pending` in `metadata.txt` records the state at export time. The imported result is computed from `verified.csv`.

File consistency checks detect mismatched or incomplete data. They do not authenticate the external result file or independently prove primality.

#### Encrypt using the verified batch

Select `5`, provide the batch directory, and choose two original indices from the displayed verified-prime list.

For the recorded batch:

```text
Main menu: 5
Batch directory: results/batch_1791902784183277849_6753
Original index for p: 32
Original index for q: 148
```

The program rejects unknown indices and repeated prime values. It also requires both selected factors to have 40 decimal digits and to be compatible with the public exponent.

Choose a message type:

```text
1. UTF-8 text
2. Non-negative integer smaller than n
```

For text, enter a line of text. For integers, enter decimal digits representing a value in the interval `[0, n)`.

The program displays the encrypted packet, recovers the message, and checks exact equality.

### Additional RSA-OAEP simulation

Option `2` generates new 1024-bit probable primes using our ChaCha20 wrapper initialized from operating-system randomness.

It constructs a 2048-bit RSA key pair and encrypts one UTF-8 message using RSA-OAEP with SHA-256.

This option:

- Uses newly generated factors rather than the saved 40-digit batch.
- Accepts up to 190 UTF-8 bytes.
- Produces a 256-byte ciphertext.
- Uses the `rsa` crate for key construction, OAEP, and blinded decryption.
- Generates a fresh key pair on each execution.

It is an additional demonstration. Option `5` is the workflow that directly uses the verified primes from the coursework batch.

## Generation algorithms

### SHA-256 Counter

Each output block hashes a domain identifier, length-prefixed seed and salt encodings, and an incrementing counter:

$$
B_i =
\operatorname{SHA256}
\left(
D \parallel L(S) \parallel S
\parallel L(T) \parallel T
\parallel C_i
\right).
$$

Here:

- $D$ identifies the generator construction.
- $S$ and $T$ are the big-endian seed and salt bytes.
- $L$ encodes a byte length as a 64-bit big-endian integer.
- $C_i$ is the 64-bit big-endian counter.
- $\parallel$ denotes concatenation.

The counter begins at zero. Each digest supplies 32 bytes.

The project implements the counter construction; the `sha2` crate supplies SHA-256. Unused bytes from a digest are discarded, so reproducibility requires the same parameters and the same pattern of byte requests.

### ChaCha20

SHA-256 derives a 32-byte key and a 12-byte nonce from two numeric seeds, using separate domain identifiers.

ChaCha20 mixes its internal state using:

- Addition modulo $2^{32}$.
- Bitwise XOR.
- Bit rotations.

Our wrapper applies the keystream to a zero-filled buffer, producing the pseudorandom bytes themselves. The `chacha20` crate retains the stream position between calls.

The original experiment uses user-provided seeds for reproducibility. The RSA-OAEP extension initializes the wrapper using fresh system-generated seeds.

### Blum Blum Shub

Blum Blum Shub is implemented directly in the project:

$$
M=pq,
$$

$$
x_0=\mathrm{seed}^2\bmod M,
$$

$$
x_{i+1}=x_i^2\bmod M.
$$

Each step emits the least significant bit of the updated state. Eight bits form one output byte.

The implementation requires:

- Distinct probable primes $p$ and $q$.
- $p\equiv q\equiv3\pmod4$.
- $1<\mathrm{seed}<pq$.
- $\gcd(\mathrm{seed},pq)=1$.
- An initial state different from the fixed state $1$.

Small parameters may produce short cycles. A 40-digit output format alone does not establish a long period or cryptographic security.

## Number conversion and primality

### Producing exactly 40 decimal digits

All three generators feed the same sampler:

1. Request 17 bytes.
2. Mask off the top three bits, leaving 133 bits.
3. Interpret the result as an integer $r$.
4. Accept only if $r<9\cdot10^{39}$.
5. Return $N=10^{39}+r$.

Therefore:

$$
10^{39}\le N<10^{40}.
$$

Every accepted number has exactly 40 decimal digits.

The rejection step avoids modulo bias when the input bits are uniform. The sampler stops with an error after 1,024 consecutive rejected candidates.

Large integers are represented with `BigUint`; no floating-point conversion is needed.

### Miller-Rabin

The primality module first handles small values and divisibility by the primes from 2 through 37.

For an odd candidate $N>37$, it decomposes:

$$
N-1=d\,2^s,
\qquad d\text{ odd}.
$$

Each round chooses an independent uniformly sampled base:

$$
2\le a\le N-2.
$$

It computes:

$$
x=a^d\bmod N.
$$

The round passes if:

$$
a^d\equiv1\pmod N
$$

or:

$$
a^{d2^r}\equiv-1\pmod N
\quad\text{for some }0\le r<s.
$$

Repeated modular squaring checks the second condition.

Failure proves compositeness. Passing all 32 rounds produces a probable-prime classification.

For a fixed odd composite, independent uniform bases give the false-acceptance bound:

$$
4^{-32}=2^{-64}.
$$

This is not a posterior probability that a particular accepted candidate is composite, nor a bound for the complete application.

| Classification | Meaning |
| --- | --- |
| `composite` / `not prime` | The primality test established compositeness for these generated numbers. |
| `probably_prime` / `probably prime` | Miller-Rabin accepted the candidate without producing a proof. |
| External `is_prime = 1` | PARI/GP reported a rigorous positive primality decision. |

The project implements the Miller-Rabin decision logic. `num-bigint` provides arithmetic and modular exponentiation; `rand` provides randomness for selecting bases.

## RSA mathematics and implementation

### Building keys from the selected batch values

Let $p$ and $q$ be two distinct verified primes selected from the batch.

The modulus is:

$$
n=pq.
$$

For two distinct primes, the Carmichael function is:

$$
\lambda(n)=\operatorname{lcm}(p-1,q-1).
$$

The implementation calculates the least common multiple using:

$$
\operatorname{lcm}(a,b)
=
\frac{a}{\gcd(a,b)}\,b.
$$

The public exponent is fixed at:

$$
e=65537.
$$

A valid pair must satisfy:

$$
\gcd(e,\lambda(n))=1.
$$

The private exponent is the modular inverse:

$$
d=e^{-1}\bmod\lambda(n).
$$

Equivalently:

$$
ed\equiv1\pmod{\lambda(n)}.
$$

`textbook.rs` implements the Euclidean algorithm for the greatest common divisor and the extended Euclidean algorithm for the inverse.

The extended algorithm finds coefficients satisfying:

$$
ue+v\lambda(n)=1.
$$

Reducing $u$ modulo $\lambda(n)$ gives $d$. Signed `BigInt` values hold intermediate coefficients; the normalized inverse is converted to `BigUint`.

The transmitter receives $(n,e)`. The receiver holds $(n,d)`. The factors are taken directly from the imported batch; this route does not generate replacement primes.

### Encrypting and decrypting integers

The message must satisfy:

$$
0\le m<n.
$$

Encryption computes:

$$
c=m^e\bmod n.
$$

Decryption computes:

$$
m'=c^d\bmod n.
$$

The application rejects message or ciphertext integers outside `[0, n)`.

The message itself does not have to be prime. A composite number, zero, or a number sharing a factor with $n$ can still be recovered correctly.

### Why recovery works

Since $ed\equiv1\pmod{\lambda(n)}$, it is also congruent to 1 modulo both $p-1$ and $q-1$.

If $m$ is not divisible by $p$, Fermat's little theorem implies:

$$
m^{ed}\equiv m\pmod p.
$$

If $m$ is divisible by $p$, both sides are zero modulo $p$, so the same congruence holds.

The corresponding argument applies modulo $q$. The Chinese remainder theorem then gives:

$$
m^{ed}\equiv m\pmod{pq}.
$$

Because the recovered integer lies in `[0, n)`, it equals the original message.

### Encoding text into blocks

Text is converted to UTF-8 bytes and split into payload blocks.

If the modulus has $k$ bits, the implementation uses:

$$
b=\left\lfloor\frac{k-1}{8}\right\rfloor-1
$$

payload bytes per block.

Each payload receives a leading byte with value `1`. The resulting byte sequence is interpreted as a big-endian integer.

This prefix preserves leading zero bytes when the integer is converted back to bytes. It is an encoding marker, not cryptographic padding or authentication.

The encoded block occupies at most $b+1$ bytes, ensuring:

$$
m<2^{8(b+1)}\le2^{k-1}\le n.
$$

Encrypted text blocks are serialized using the fixed width:

$$
w=\left\lceil\frac{k}{8}\right\rceil
$$

bytes. Leading zero bytes are added to ciphertext blocks when necessary to preserve their boundaries.

The receiver:

1. Checks that the packet contains complete ciphertext blocks.
2. Decrypts each block.
3. Checks and removes the prefix.
4. Reassembles all payload bytes.
5. Decodes the complete result as UTF-8.

Decoding after reassembly allows a multibyte character to span block boundaries.

An empty text message produces an empty packet and is recovered as an empty string.

For the recorded 265-bit modulus, each text block carries 32 payload bytes and occupies 34 ciphertext bytes. Integer mode instead sends one integer using its big-endian byte representation, which may be shorter.

### Educational RSA limitations

This route deliberately demonstrates the RSA equations using the 40-digit factors from the assignment.

It does not use OAEP and is deterministic. Equal encoded blocks under the same key produce equal ciphertext blocks.

The encoding checks do not provide authentication. Complete blocks can potentially be removed, rearranged, or replaced without those checks detecting every modification.

The factors are also recorded in the experiment files. This workflow is intended for explanation and testing, not for protecting private information.

### The additional RSA-OAEP route

The separate OAEP workflow generates odd candidates with their two highest bits set:

$$
3\cdot2^{1022}\le p,q<2^{1024}.
$$

Their product therefore has exactly 2048 bits:

$$
2^{2047}<9\cdot2^{2044}\le pq<2^{2048}.
$$

The search also requires:

- Acceptance by the 32-round Miller-Rabin test.
- Compatibility with $e=65537$.
- A factor distance greater than $2^{924}$.

It stops after 100,000 generated candidates if no suitable pair is found.

The `rsa` crate constructs the key from those supplied factors. OAEP uses a fresh random seed, SHA-256, and mask generation to encode the message before applying RSA.

For a 256-byte modulus and a 32-byte hash output, its message limit is:

$$
\mathrm{mLen}\le256-2(32)-2=190\text{ bytes}.
$$

Each ciphertext occupies 256 bytes. The receiver uses blinded decryption and validates the OAEP encoding.

This route continues to use probable primes without external confirmation. Its key size and use of OAEP do not constitute a security audit of the application.

## Transmitter, channel, and receiver

Both encryption demonstrations use logical components within one process.

| Component | Information held | Responsibility |
| --- | --- | --- |
| Transmitter | Public key and outgoing message | Encode and encrypt the message. |
| Channel | Ciphertext packets | Store and deliver packets in first-in, first-out order. |
| Receiver | Private key and incoming ciphertext | Decrypt and reconstruct the message. |
| Coordinator | Original and recovered messages | Display the demonstration and check exact recovery. |

The channel uses `VecDeque<Vec<u8>>`. It receives neither plaintext nor keys.

The demonstrations do not use sockets or separate computers. The components are not isolated operating-system processes.

`Exact message match: yes` reports a local equality check. It is not a digital signature or sender authentication.

## Experimental results

### Original 40-digit experiments

The following historical runs used release builds and 100,000 numbers per algorithm.

| Algorithm | Probable primes | Composites | Probable-prime percentage | Time |
| --- | ---: | ---: | ---: | ---: |
| SHA-256 Counter | 1,062 | 98,938 | 1.0620% | 0.729 s |
| ChaCha20 | 1,080 | 98,920 | 1.0800% | 0.760 s |
| Blum Blum Shub | 1,103 | 98,897 | 1.1030% | 2.554 s |

Parameters:

| Algorithm | Configuration |
| --- | --- |
| SHA-256 Counter | `seed = 109281`, `salt = 102939` |
| ChaCha20 | `key_seed = 109281`, `nonce_seed = 102939` |
| Blum Blum Shub | `p = 9223372036854876599`, `q = 9223372036854989879`, `seed = 101` |

These runs produced 300,000 numbers, including 3,245 probable-prime classifications.

The reported development machine used Debian, an AMD Ryzen 7 8745HX, and approximately 31 GiB of RAM.

Batch timing includes allocation, generation, primality testing, and result storage. It excludes compilation, user input, generator initialization, BBS parameter validation, and printing.

### Verified coursework batch

The recorded coursework batch is:

```text
batch_1791902784183277849_6753
```

| Property | Result |
| --- | --- |
| Generator | SHA-256 Counter |
| Seed | `12908389213` |
| Salt | `12783665722` |
| Generated numbers | 10,000 |
| Decimal digits per number | 40 |
| Miller-Rabin candidates | 107 |
| Candidates confirmed prime by PARI | 107 |
| Candidates rejected by PARI | 0 |
| Total composites | 9,893 |
| Prime percentage | 1.0700% |
| Generation and Miller-Rabin time | 0.075117 s |

The recorded time excludes external PARI verification and file export.

All 107 external results matched the original indices and values. This confirms the classifications for this batch; it does not make Miller-Rabin a definitive primality test for future batches.

### RSA using primes from the coursework batch

The demonstration selected:

| Original index | Verified prime |
| ---: | --- |
| 32 | `8226342136733210179679450416358439885889` |
| 148 | `4319206805826234237966605867347973035363` |

The resulting public modulus was:

```text
35531272944033207404163099118541722910330323451210634734096018386858962681692707
```

Its size was 265 bits, with public exponent `65537`.

The numeric message was the final number of the original batch:

```text
3207310812540536705408775844862896298045
```

Its ciphertext integer was:

```text
20453079798678437144204544286815934259840760416779309595885140617531901163790109
```

The receiver recovered:

```text
3207310812540536705408775844862896298045
```

The application reported:

```text
Exact message match: yes
The RSA factors came from the verified batch.
```

### Additional RSA-OAEP demonstration

One recorded release-mode execution produced:

| Measurement | Result |
| --- | ---: |
| Candidates generated | 70 |
| Factor sizes | 1024 bits each |
| Modulus size | 2048 bits |
| Factor generation and testing | 0.061 s |
| Original UTF-8 message size | 45 bytes |
| Ciphertext size | 256 bytes |
| Encryption | 0.236 ms |
| Decryption | 2.064 ms |
| Exact recovery | Yes |

All timings are observations from individual executions, not benchmark guarantees.

More primes do not imply a better generator. Similar prime counts do not prove uniformity or cryptographic security.

## Testing and project structure

Run:

```bash
cargo test --release
```

The recorded current version passed **24 tests**.

| Area | Tests |
| --- | ---: |
| Original generators, sampler, primality, and experiment logic | 14 |
| RSA-OAEP key construction and communication | 2 |
| Coursework export | 1 |
| External verification import | 3 |
| Educational RSA | 4 |
| Total | 24 |

Coverage includes:

- Exactly 40 decimal digits per generated number.
- Reproducibility and parameter changes.
- BBS parameter validation and reference output.
- Generator buffer handling.
- Miller-Rabin cases and small-number comparisons.
- Preservation of experiment order.
- Preservation of exported indices and integer values.
- Rejection of missing, duplicated, changed, and foreign verification results.
- Exclusion of candidates rejected by external verification.
- Construction of RSA-OAEP keys from the supplied factors.
- OAEP round trips, byte limits, and invalid ciphertext rejection.
- Educational RSA recovery of zero, ordinary integers, and values not coprime to the modulus.
- Empty text, zero bytes, whitespace, and multiblock Unicode.
- Rejection of invalid factor selections, out-of-range integers, and incomplete text blocks.

These tests verify specific behaviors. They do not constitute a security audit or independently validate the imported PARI decisions.

### Source structure

| File | Purpose |
| --- | --- |
| `src/main.rs` | Main menus and original experiment interface. |
| `src/experiment.rs` | Batch generation, classifications, counts, and timing. |
| `src/number.rs` | Shared sampler for 40-digit numbers. |
| `src/prime.rs` | Miller-Rabin and supporting functions. |
| `src/generators/mod.rs` | Common generator interface. |
| `src/generators/sha256.rs` | SHA-256 Counter generator. |
| `src/generators/chacha20.rs` | ChaCha20 wrapper. |
| `src/generators/bbs.rs` | Blum Blum Shub implementation. |
| `src/cryptography/mod.rs` | RSA-OAEP workflow and module declarations. |
| `src/cryptography/primes.rs` | Generation of 1024-bit probable-prime factors. |
| `src/cryptography/keys.rs` | RSA-OAEP key construction from those factors. |
| `src/cryptography/transmitter.rs` | OAEP encryption and message-length checks. |
| `src/cryptography/channel.rs` | Shared in-memory ciphertext queue. |
| `src/cryptography/receiver.rs` | Blinded OAEP decryption and UTF-8 recovery. |
| `src/cryptography/coursework.rs` | Generation and export of 10,000-number batches. |
| `src/cryptography/verified.rs` | Import and consistency checks for external results. |
| `src/cryptography/textbook.rs` | Educational RSA mathematics and text encoding. |
| `src/cryptography/educational.rs` | Interactive RSA simulation using the verified batch. |

Cryptography-specific logic remains grouped under `src/cryptography/`. The original generators and primality module are reused.

## Scope

The assignment requirements are addressed through:

| Requirement | Implementation |
| --- | --- |
| Parameterized generation of 40-digit numbers | Three configurable generators and the shared sampler. |
| At least three techniques | SHA-256 Counter, ChaCha20, and Blum Blum Shub. |
| Generation of 10,000 numbers | Coursework batch mode. |
| Identification of primes | Miller-Rabin filtering followed by external PARI verification. |
| Encryption using the primes found | Educational RSA using selected indices from the verified batch. |
| Encryption tests | Automated tests and a recorded numeric round trip. |

Users control generator parameters, experiment size, batch selection, RSA-factor indices, and messages. Saved results preserve the connection between generation, verification, and encryption.

Current boundaries include:

- PARI verification is external and its reported results are trusted after consistency checks.
- No mathematical primality certificates are imported or checked by Rust.
- Educational RSA uses small, publicly recorded factors and no OAEP.
- The OAEP extension uses separately generated probable-prime factors.
- Communication is simulated within one process.
- No sender authentication, signatures, trusted key distribution, or replay protection.
- No persistent private-key storage.
- Sequential execution without GPU acceleration.
- Classified batch results remain in memory until printing.

The project demonstrates the mathematics and implementation of the assignment. It is not an audited production cryptographic system.

## References

- [RFC 8017: RSA primitives and RSAES-OAEP](https://www.rfc-editor.org/rfc/rfc8017.html).
- [RustCrypto RSA](https://github.com/RustCrypto/RSA).
- [PARI/GP arithmetic functions, including isprime](https://pari.math.u-bordeaux.fr/dochtml/html-stable/Arithmetic_functions.html).
- [Handbook of Applied Cryptography, Chapter 4: primality testing](https://cacr.uwaterloo.ca/hac/about/chap4.pdf).