"""sac22cc3jp4 generator: `from s to` lines, s a positive 32-bit value in base `from`.

Targets stay at most base 10 (assumed statement bound): the accepted C++ prints
each digit with to_string, so its output above base 10 is not a numeral.
"""
import random
import string
import sys

DIGITS = string.digits + string.ascii_uppercase
seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
q = rng.randint(1, max(1, min(size, 1000)))
print(q)
for _ in range(q):
    b1, b2 = rng.randint(2, 36), rng.randint(2, 10)
    v = rng.randint(1, rng.choice([b1, 1000, 2**31 - 1]))
    s = ""
    while v:
        s, v = DIGITS[v % b1] + s, v // b1
    print(b1, s, b2)
