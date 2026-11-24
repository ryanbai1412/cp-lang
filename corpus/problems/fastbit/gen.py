"""gen.py SEED SIZE | --max: Fast Bit Counting, adapted to stdin as `N S`: setbits is called on
the N outputs of splitmix64 seeded with S (0 <= S < 2^64), and the answer is their sum.
N <= 1e8 calls. --max is N = 1e8."""
import random, sys
if sys.argv[1] == "--max":
    print(10**8, 2**64 - 1)
    sys.exit()
r = random.Random(int(sys.argv[1]))
print(r.randint(0, min(10**8, int(sys.argv[2]))), r.choice([0, 2**64 - 1, r.randrange(2**64)]))
