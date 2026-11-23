"""gen.py seed size: COCI '21 C2 P5. n, q <= 2e5; suspect i has heights lo..=hi in 1..=1e9;
queries l..=r with 1 <= l <= r <= n. Height ranges come from a small or a wide span."""
import random, sys

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n, q = rng.randint(1, max(1, min(200_000, size))), rng.randint(1, max(1, min(200_000, size)))
top = rng.choice([10, 1000, 10 ** 9])
out = [str(n)]
for _ in range(n):
    lo = rng.randint(1, top)
    out.append(f"{lo} {rng.randint(lo, min(top, lo + rng.choice([0, 3, top])))}")
out.append(str(q))
for _ in range(q):
    l = rng.randint(1, n)
    out.append(f"{l} {rng.randint(l, n)}")
print("\n".join(out))
