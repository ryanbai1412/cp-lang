"""mccc5j3 generator: N K M, then N distinct a and N distinct b in 1..=M; many b pair with an
a. --max: N = 2e5, M = 1e18."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 200_000 if big else r.randint(1, 1 + size * 2)
m = 10**18 if big else r.randint(2 * n + 2, 2 * n + 2 + size * 4)
k = r.randint(0, n - 1)
a = r.sample(range(1, m), n) if not big else [r.randint(1, m - 1) for _ in range(n)]
a = list(dict.fromkeys(a))
while len(a) < n:
    x = r.randint(1, m - 1)
    if x not in a:
        a.append(x)
want = [m - x for x in a if r.random() < 0.7]
pool = set(want)
b = list(dict.fromkeys(want))
while len(b) < n:
    x = r.randint(1, m - 1)
    if x not in pool:
        pool.add(x)
        b.append(x)
b = b[:n]
r.shuffle(b)
print(n, k, m)
print(*a)
print(*b)
