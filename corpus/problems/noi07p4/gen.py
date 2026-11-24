"""noi07p4 (Necklace Factory) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = q = 5e5).

"n c", n colours in 1..=c, q, then commands R k / F / S i j / P i j x / C / CS i j with
1 <= k < n. --max: c = 1000 and a mix that rotates and flips between paints and wrapping
segment counts.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 5 * 10**5 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
q = size if MAX else rng.randint(1, size)
c = 1000 if MAX else rng.choice([1, 2, 5, 1000])
out = [f"{n} {c}", " ".join(str(rng.randint(1, c)) for _ in range(n)), str(q)]
ops = ["R", "F", "S", "P", "C", "CS"]
for _ in range(q):
    op = rng.choice(ops if n > 1 else ["F", "S", "P", "C", "CS"])
    if op == "R":
        out.append(f"R {rng.randint(1, n - 1)}")
    elif op in ("F", "C"):
        out.append(op)
    elif op == "S":
        out.append(f"S {rng.randint(1, n)} {rng.randint(1, n)}")
    elif op == "P":
        out.append(f"P {rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, c)}")
    else:
        out.append(f"CS {rng.randint(1, n)} {rng.randint(1, n)}")
print("\n".join(out))
