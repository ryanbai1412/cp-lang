"""coci20c6p3 generator: `n k`, then n distinct words of 1..10 lowercase letters.

Words are shuffles of a few base letter multisets, so anagram classes repeat and grow large;
k is 0, small (subtask 2), or anywhere up to 2000.
"""
import random
import string
import sys

# --max: n = k = 2000, words of 10 letters: anagram classes of 2, 3, ... words (about 1000 in all)
# plus 1000 random words, so the knapsack runs every class over all of 0..=k.
if sys.argv[1] == "--max":
    rng, words, seen = random.Random(0), [], set()
    for size in range(2, 63):
        if len(words) + size > 1000:
            break
        base = "".join(rng.sample(string.ascii_lowercase, 10))
        cls = set()
        while len(cls) < size:
            cls.add("".join(rng.sample(base, 10)))
        words += sorted(cls); seen |= cls
    while len(words) < 2000:
        w = "".join(rng.choice(string.ascii_lowercase) for _ in range(10))
        if w not in seen:
            seen.add(w); words.append(w)
    rng.shuffle(words)
    print(2000, 2000)
    print("\n".join(words))
    sys.exit()
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
