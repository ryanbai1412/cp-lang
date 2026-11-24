"""ioi15p1 generator: `n k l`, then n sorted positions in 0..l-1 (repeats allowed)."""
import random
import sys

if sys.argv[1] == "--max":
    r = random.Random(0)
    n, k, l = 10_000_000, 1000, 1_000_000_000
    print(n, k, l)
    sys.stdout.write(" ".join(map(str, sorted(r.randrange(l) for _ in range(n)))) + "\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = r.randint(1, max(1, size))
k = r.choice([1, n, r.randint(1, n)])
l = r.choice([1, 2, r.randint(1, 2 * size + 2), r.randint(1, 10**9)])
print(n, k, l)
print(*sorted(r.randrange(l) for _ in range(n)))
