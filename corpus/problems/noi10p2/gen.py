"""noi10p2 generator: `n k L R`, then n notes in -1000..=1000; k <= min(5e5, number of chords).

`python3 gen.py SEED SIZE [MODE]`: MODE `neg` makes every note negative, `wide` takes L=1, R=n,
`max` uses n = SIZE with k at its largest, `narrow` is `max` with R - L <= 2.
"""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
mode = sys.argv[3] if len(sys.argv) > 3 else ""
rng = random.Random(seed)
n = size if mode in ("max", "narrow") else rng.randint(1, max(1, size))
lo = rng.randint(1, n)
hi = rng.randint(lo, n)
if mode == "wide":
    lo, hi = 1, n
if mode == "narrow":
    hi = min(n, lo + rng.randint(0, 2))
chords = sum(n - m + 1 for m in range(lo, hi + 1))
cap = min(chords, 500_000)
print(n, cap if mode in ("max", "narrow") else rng.randint(1, cap), lo, hi)
top = -1 if mode == "neg" else 1000
print(*(rng.randint(-1000, top) for _ in range(n)))
