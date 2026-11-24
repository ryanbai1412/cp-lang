"""ioi17p4 generator: n, then the hidden kinds of the n boxes: one diamond (kind 0) and more
than k^2 boxes of kind t + 1 when kind t has k."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 200_000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 10)
n = size if big else r.randint(3, max(3, size))
counts = [1]
while True:
    nxt = counts[-1] ** 2 + 1 + r.randint(0, counts[-1] + 2)
    rest = n - sum(counts) - nxt
    if rest <= nxt * nxt or r.random() < 0.2:
        break
    counts.append(nxt)
counts.append(n - sum(counts))
kinds = [t for t, c in enumerate(counts) for _ in range(c)]
r.shuffle(kinds)
print(n)
print(*kinds)
