from generators import PseudoRandomGenerator

RANDOM_BYTES = 17
MAX_REJECTION_ATTEMPTS = 1024
MIN_40_DIGITS = 10**39
RANGE_40_DIGITS = 9 * MIN_40_DIGITS


def generate_40_digit_number(generator: PseudoRandomGenerator) -> int:
    for _ in range(MAX_REJECTION_ATTEMPTS):
        raw = bytearray(generator.fill_bytes(RANDOM_BYTES))
        # Keep the same 133-bit candidate space as the Rust implementation.
        raw[0] &= 0b0001_1111
        candidate = int.from_bytes(raw, byteorder="big")

        # Rejection sampling avoids modulo bias.
        if candidate < RANGE_40_DIGITS:
            return MIN_40_DIGITS + candidate

    raise RuntimeError(
        "Could not generate a 40-digit number after "
        f"{MAX_REJECTION_ATTEMPTS} attempts. Try different generator parameters."
    )
