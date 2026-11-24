"""ioi13p6 (Game): `gen.py SEED SIZE`, or `gen.py --max`.

"R C N", then N actions: "1 P Q K" sets cell (P, Q) to K, "2 P Q U V" asks the GCD of the
rectangle with corners (P, Q) and (U, V) (1 <= R, C <= 1e9, at most 22000 updates and
250000 questions, 0 <= K <= 1e18). The output is one GCD per question.
--max: R = C = 1e9, 22000 updates of 60-bit multiples of small primes, 250000
questions over wide rectangles, the two interleaved.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    r = c = 1_000_000_000
    nu, nq = 22_000, 250_000
else:
    size = max(1, int(sys.argv[2]))
    r, c = rng.choice([1, 3, size, 1_000_000_000]), rng.choice([1, 3, size, 1_000_000_000])
    nu, nq = rng.randint(1, min(22_000, size)), rng.randint(1, min(250_000, size))
rows = [rng.randrange(r) for _ in range(min(r, 200))]
cols = [rng.randrange(c) for _ in range(min(c, 200))]
base = rng.choice([1, 2, 6, 30])


def value():
    k = rng.randint(0, 10**18 // base) * base
    return 0 if rng.random() < 0.05 else k


acts = ["1"] * nu + ["2"] * nq
rng.shuffle(acts)
acts.sort(key=lambda a: a == "2" and rng.random() < 0.02)
out = [f"{r} {c} {nu + nq}"]
for a in acts:
    if a == "1":
        p = rng.choice(rows) if rng.random() < 0.5 else rng.randrange(r)
        q = rng.choice(cols) if rng.random() < 0.5 else rng.randrange(c)
        out.append(f"1 {p} {q} {value()}")
    else:
        p, u = sorted((rng.randrange(r), rng.randrange(r)))
        q, v = sorted((rng.randrange(c), rng.randrange(c)))
        if MAX:
            p, q, u, v = p // 8, q // 8, r - 1 - (r - 1 - u) // 8, c - 1 - (c - 1 - v) // 8
        out.append(f"2 {p} {q} {u} {v}")
print("\n".join(out))
