"""gen.py SEED SIZE | --max: CCC 2003 S4. A count N of strings, each 1 to 5000 alphanumeric characters.
Shapes: random over a small or the full alphabet, one repeated letter, a short period.
--max is 5 strings of 5000 random characters."""
import random, string, sys

MAX = sys.argv[1] == "--max"
seed, size = (0, 5000) if MAX else (int(sys.argv[1]), int(sys.argv[2]))
rng = random.Random(seed)
full = string.ascii_letters + string.digits
out = []
for _ in range(5 if MAX else rng.randint(1, 5)):
    n = 5000 if MAX else rng.randint(1, max(1, min(5000, size)))
    shape = "full" if MAX else rng.choice(["small", "full", "one", "period"])
    if shape == "one":
        s = rng.choice(full) * n
    elif shape == "period":
        p = "".join(rng.choice(full) for _ in range(rng.randint(1, 4)))
        s = (p * n)[:n]
    else:
        alpha = "abc" if shape == "small" else full
        s = "".join(rng.choice(alpha) for _ in range(n))
    out.append(s)
print(len(out))
print("\n".join(out))
