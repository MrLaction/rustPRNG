from generators import BlumBlumShub, ChaCha20Generator, PseudoRandomGenerator, Sha256Counter
from number import generate_40_digit_number


def read_line(prompt: str) -> str:
    return input(prompt).strip()


def read_biguint(prompt: str) -> int:
    while True:
        value = read_line(prompt)
        if value and value.isascii() and value.isdigit():
            return int(value)
        print("Invalid input. Please enter a non-negative integer.")


def read_usize(prompt: str) -> int:
    while True:
        value = read_line(prompt)
        if value and value.isascii() and value.isdigit() and int(value) > 0:
            return int(value)
        print("Please enter a positive integer.")


def select_generator() -> PseudoRandomGenerator:
    while True:
        print("\nPseudorandom Number Generator")
        print("\nSelect an algorithm:")
        print("1. SHA-256")
        print("2. ChaCha20")
        print("3. Blum Blum Shub")

        option = read_line("> ")

        if option == "1":
            print("\nSHA-256 selected.\n")
            seed = read_biguint("Seed: ")
            salt = read_biguint("Salt: ")
            return Sha256Counter(seed, salt)

        if option == "2":
            print("\nChaCha20 selected.\n")
            key_seed = read_biguint("Key seed: ")
            nonce_seed = read_biguint("Nonce seed: ")
            return ChaCha20Generator(key_seed, nonce_seed)

        if option == "3":
            print("\nBlum Blum Shub selected.")
            print("p and q must be distinct probable primes.")
            print("Both must satisfy value % 4 == 3.\n")
            p = read_biguint("p: ")
            q = read_biguint("q: ")
            seed = read_biguint("Seed: ")
            try:
                return BlumBlumShub(p, q, seed)
            except ValueError as error:
                print(f"Invalid BBS parameters: {error}")
            continue

        print("Invalid option.")


def main() -> None:
    generator = select_generator()
    print(f"\nAlgorithm: {generator.name()}")
    count = read_usize("How many numbers do you want to generate?: ")
    print()

    for index in range(1, count + 1):
        number = generate_40_digit_number(generator)
        print(f"{index:>5}: {number}")


if __name__ == "__main__":
    main()
