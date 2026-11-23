"""ioi96p1 generator: python3 gen.py SEED [SIZE]. n in 1..=min(SIZE, 100), numbers in 0..=200."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
cap = max(1, min(size, 100))
n = rng.choice([rng.randint(1, cap), cap])
hi = rng.choice([1, 10, 200])
mode = rng.choice(["random", "random", "equal", "max"])
a = [rng.randint(0, hi) for _ in range(n)]
if mode == "equal":
    a = [hi] * n
elif mode == "max":
    a = [rng.randint(190, 200) for _ in range(n)]
print(n)
print(*a)
