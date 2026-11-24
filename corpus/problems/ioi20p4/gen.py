"""ioi20p4 generator: q, then q calls `k x` and a[0..k) with total tastiness sum a[i] 2^i <= 1e18."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 6
q = 1000 if big else r.randint(1, max(1, size))
print(q)
for _ in range(q):
    k = 60 if big else r.randint(1, min(60, max(1, 2 * size)))
    huge = big or r.random() < 0.3
    room = 10**18
    a = []
    for i in range(k):
        v = r.randint(0, (room // k >> i) if huge else 6)
        a.append(v)
    x = r.randint(1, 10**18) if huge and r.random() < 0.3 else r.randint(1, 6)
    print(k, x)
    print(*a)
