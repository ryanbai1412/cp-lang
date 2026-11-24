"""ioi08p3 (Fish) inputs: `gen.py SEED SIZE`, or `gen.py --max` (F = 5e5).

F, K, M, then F lines "L G" (1 <= L <= 1e9, every kind 1..=K on some fish). --max: K = 1e5
kinds over 5e5 fish, lengths over many doublings so each kind's window takes many fish,
M = 30000.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 5 * 10**5 if MAX else max(1, int(sys.argv[2]))
f = size if MAX else rng.randint(1, size)
k = 10**5 if MAX else rng.randint(1, f)
m = 30000 if MAX else rng.randint(2, 30000)
hi = 10**9 if MAX else rng.choice([4, 30, 10**9])
gems = list(range(1, k + 1)) + [rng.randint(1, k) for _ in range(f - k)]
rng.shuffle(gems)
out = [str(f), str(k), str(m)]
for g in gems:
    length = int(2 ** rng.uniform(0, 29.89)) if MAX else rng.randint(1, hi)
    out.append(f"{max(1, min(hi, length))} {g}")
print("\n".join(out))
