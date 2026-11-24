"""ioi17p3 generator: `n m`, owners, charges, then m tracks `u v`; every station has a track out."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 5000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 6)
n = size if big else r.randint(1, max(1, size))
m = 20_000 if big else r.randint(n, max(n, min(20_000, 3 * n)))
charge_p = 0.05 if big else r.random()
tracks = [(v, r.randrange(n)) for v in range(n)]
tracks += [(r.randrange(n), r.randrange(n)) for _ in range(m - n)]
r.shuffle(tracks)
print(n, m)
print(*(r.randint(0, 1) for _ in range(n)))
print(*(int(r.random() < charge_p) for _ in range(n)))
for u, v in tracks:
    print(u, v)
