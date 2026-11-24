"""ioi05p1 (Garden) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"L W", then N, then K, then N roses "r c" (1 <= r <= L, 1 <= c <= W, repeats allowed).
--max: L = W = 250, N = 5000 roses spread over the garden, K = 10 so many rectangles count.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    l = w = 250
    n, k = 5000, 10
else:
    size = max(2, int(sys.argv[2]))
    l, w = rng.randint(1, min(250, size)), rng.randint(1, min(250, size))
    n = rng.randint(2, min(5000, size))
    k = rng.randint(1, max(1, n // 2))
print(f"{l} {w}")
print(n)
print(k)
print("\n".join(f"{rng.randint(1, l)} {rng.randint(1, w)}" for _ in range(n)))
