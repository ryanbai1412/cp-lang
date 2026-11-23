"""gen.py seed size: CCC 2003 S4. A count N of strings, each 1 to 5000 alphanumeric characters.
Shapes: random over a small or the full alphabet, one repeated letter, a short period."""
import random, string, sys

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
full = string.ascii_letters + string.digits
out = []
for _ in range(rng.randint(1, 5)):
    n = rng.randint(1, max(1, min(5000, size)))
    shape = rng.choice(["small", "full", "one", "period"])
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
