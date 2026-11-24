"""gen.py SEED SIZE | --max: CCC 2019 S3. A square v = a + i p + j q + i j s (every square with
arithmetic rows and columns has this form), values in [-1e6, 1e6], with some cells hidden.
--max hides all nine cells and uses the widest values on the given ones' side."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
B = 10 ** 6 if MAX else r.choice([5, 50, 10 ** 5])
while True:
    a, p, q, s = (r.randint(-B, B) for _ in range(4))
    sq = [a + i * p + j * q + i * j * s for i in range(3) for j in range(3)]
    if all(abs(v) <= 10 ** 6 for v in sq):
        break
hide = set(range(9)) if MAX else set(r.sample(range(9), r.randint(0, 9)))
for i in range(3):
    print(*["X" if 3 * i + j in hide else sq[3 * i + j] for j in range(3)])
