"""nccc9s4 generator: n q, the n scores in 1..=n, then q queries l r. --max: n = q = 1e5,
about 2000 distinct scores, half the queries long."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 100_000 if big else r.randint(1, 1 + size * 40)
q = 100_000 if big else r.randint(1, 1 + size * 10)
top = 2000 if big else r.randint(1, n)
print(n, q)
print(*(r.randint(1, top) for _ in range(n)))
for _ in range(q):
    if r.random() < 0.5:
        l = r.randint(1, n)
        print(l, r.randint(l, min(n, l + 300)))
    else:
        l, rr = sorted((r.randint(1, n), r.randint(1, n)))
        print(l, rr)
