"""gen.py seed size: SAC '22 CC3 P3. 1 <= N <= 1e5 distinct 1 <= A_i <= 1e9.
size >= 1e5 gives N = 1e5 with shapes by seed: uniform to 1e9 (10 rounds); all ten-digit
values; values sharing long suffixes (fewer than N when the range is small); mixed lengths 1..1e9."""
import random, sys
if sys.argv[1] == "--max":
    sys.argv[1:] = ["1", "100000"]
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 10 ** 5
n = 10 ** 5 if big else rng.randint(1, size + 1)
shape = seed % 4 if big else rng.randrange(4)
hi = 10 ** 9 if big or rng.random() < 0.5 else rng.choice([9, 100, 10 ** 4])
seen = set()
tries = 0
while len(seen) < min(n, hi) and tries < 50 * n:
    tries += 1
    if shape == 0: x = rng.randint(1, hi)
    elif shape == 1: x = rng.randint(max(1, hi // 10), hi)
    elif shape == 2: x = min(hi, rng.randint(1, 10 ** rng.randint(0, 6)) * 1000 + rng.choice([1, 7, 123])) if hi > 1000 else rng.randint(1, hi)
    else: x = min(hi, rng.randint(1, 10 ** rng.randint(1, 9)))
    seen.add(x)
a = list(seen); rng.shuffle(a)
print(len(a)); print(" ".join(map(str, a)))
