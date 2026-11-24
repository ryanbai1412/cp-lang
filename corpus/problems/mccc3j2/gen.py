"""gen.py SEED SIZE | --max: Mock CCC '22 C1 J2. N, K <= 1e4 and a U/D/F trail of length N.
Small K and an uphill-heavy trail make Bob walk often; --max is N = 1e4 with K = 1e4 / 2
over a trail that drops him to speed 0 halfway and keeps him near it."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10_000 if MAX else r.randint(1, min(10_000, 2 + 3 * S))
k = n // 2 if MAX else r.randint(1, min(10_000, 1 + S))
trail = "U" * (n // 2) + "".join(r.choice("UUDF") for _ in range(n - n // 2)) if MAX else \
    "".join(r.choice(r.choice(["UDF", "UUDF", "UUUDF"])) for _ in range(n))
print(n, k)
print(trail)
