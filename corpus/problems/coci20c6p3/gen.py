"""coci20c6p3 generator: `n k`, then n distinct words of 1..10 lowercase letters.

Words are shuffles of a few base letter multisets, so anagram classes repeat and grow large;
k is 0, small (subtask 2), or anywhere up to 2000.
"""
import random
import string
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, min(size, 2000)))
alpha = string.ascii_lowercase[: rng.choice([2, 3, 5, 26])]
bases = [
    "".join(rng.choice(alpha) for _ in range(rng.randint(1, 10)))
    for _ in range(rng.randint(1, max(1, n // rng.choice([1, 4, 50, 2000]))))
]
words: list[str] = []
seen: set[str] = set()
for _ in range(20 * n):
    if len(words) == n:
        break
    b = rng.choice(bases)
    w = "".join(rng.sample(b, len(b)))
    if w not in seen:
        seen.add(w)
        words.append(w)
while len(words) < n:
    w = "".join(rng.choice(string.ascii_lowercase) for _ in range(rng.randint(1, 10)))
    if w not in seen:
        seen.add(w)
        words.append(w)
rng.shuffle(words)
k = rng.choice([0, rng.randint(1, 3), rng.randint(0, 2000), rng.randint(0, min(2000, n * (n - 1) // 2))])
print(n, k)
print("\n".join(words))
