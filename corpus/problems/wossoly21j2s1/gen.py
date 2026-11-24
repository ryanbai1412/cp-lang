"""wossoly21j2s1 inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 1000).

Ten cipher characters (letters or digits, for 0-9), N (1 <= N <= 1000), then N digits.
--max: N = 1000.
"""
import random
import string
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 1000 if MAX else rng.randint(1, min(1000, max(1, int(sys.argv[2]))))
print(*rng.choices(string.ascii_letters + string.digits, k=10))
print(n)
print(*(rng.randint(0, 9) for _ in range(n)))
