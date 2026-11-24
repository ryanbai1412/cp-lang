"""ioi22p4 generator: n m q, the n + m parents (-1 for gate 0; gates are 0..n, every gate has
an input and P[i] < i), the m source states, then q toggles `l r` with n <= l <= r < n + m."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 100_000 if big else r.randint(1, max(1, size))
chain = r.random() < 0.2
gate_parent = [-1] + [i - 1 if chain else r.randrange(i) for i in range(1, n)]
bare = sorted(set(range(n)) - set(gate_parent))
m = 100_000 if big else len(bare) + r.randint(0, max(1, size))
q = 100_000 if big else r.randint(0, max(1, 3 * size))
source_parent = bare + [r.randrange(n) for _ in range(m - len(bare))]
r.shuffle(source_parent)
print(n, m, q)
print(*(gate_parent + source_parent))
print(*(r.randint(0, 1) for _ in range(m)))
for _ in range(q):
    l = r.randrange(n, n + m)
    print(l, r.randrange(l, n + m) if big or r.random() < 0.7 else l)
