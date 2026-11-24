"""ioi20p6 generator: n, a random tree's n - 1 links `u v`, q, then q queries `s t` (s != t)."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 1000 if big else r.randint(2, max(2, size))
q = 100_000 if big else r.randint(1, max(1, 3 * size))
label = list(range(n))
r.shuffle(label)
path = r.random() < 0.2
print(n)
for v in range(1, n):
    p = v - 1 if path else r.randint(max(0, v - (3 if r.random() < 0.5 else v)), v - 1)
    print(label[p], label[v])
print(q)
for _ in range(q):
    s, t = r.sample(range(n), 2)
    print(s, t)
