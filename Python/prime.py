import secrets

MILLER_RABIN_ROUNDS = 32
SMALL_PRIMES = (2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37)


def is_probably_prime(number: int) -> bool:
    """Return whether number passes 32 randomized Miller–Rabin rounds."""
    if number < 2:
        return False

    for prime in SMALL_PRIMES:
        if number == prime:
            return True
        if number % prime == 0:
            return False

    number_minus_one = number - 1
    d = number_minus_one
    s = 0
    while d & 1 == 0:
        d >>= 1
        s += 1

    # This samples bases uniformly from [2, number - 2], as in the Rust code.
    for _ in range(MILLER_RABIN_ROUNDS):
        base = secrets.randbelow(number - 3) + 2
        x = pow(base, d, number)

        if x == 1 or x == number_minus_one:
            continue

        for _ in range(1, s):
            x = (x * x) % number
            if x == number_minus_one:
                break
            if x == 1:
                return False
        else:
            return False

    return True
