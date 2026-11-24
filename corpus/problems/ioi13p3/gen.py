"""IOI 2013 P3 (Wombats): `R C`, R rows of C - 1 horizontal counts, R - 1 rows of C vertical
counts, then E events: `1 P Q W` (horizontal (P, Q) := W), `2 P Q W` (vertical), `3 V1 V2`
(escape). R <= 5000, C <= 200, counts < 1000, at most 500 changes and 2e5 escapes.

`gen.py SEED SIZE`: R and C up to about SIZE. `gen.py --max`: R = 5000, C = 200, 500 changes
spread over all rows among 2e5 escapes, so every change rebuilds a leaf and its path.
"""
import random
import sys

if sys.argv[1:] == ["--max"]:
    g = random.Random(1)
    rows, cols, changes, escapes, hi = 5000, 200, 500, 200_000, 999
else:
    g = random.Random(int(sys.argv[1]))
    size = max(2, int(sys.argv[2]))
    rows, cols = g.randint(1, min(5000, size)), g.randint(1, min(200, size))
    changes, escapes, hi = g.randint(0, min(500, size)), g.randint(1, size), g.choice([3, 999])
out = [f"{rows} {cols}"]
out += [" ".join(str(g.randint(0, hi)) for _ in range(cols - 1)) for _ in range(rows)]
out += [" ".join(str(g.randint(0, hi)) for _ in range(cols)) for _ in range(rows - 1)]
evs = ["c"] * changes + ["e"] * escapes
g.shuffle(evs)
if cols == 1 and rows == 1:
    evs = ["e"] * escapes
out.append(str(len(evs)))
for e in evs:
    kind = 3 if e == "e" else 1 if rows == 1 or (cols > 1 and g.random() < 0.5) else 2
    if kind == 3:
        out.append(f"3 {g.randrange(cols)} {g.randrange(cols)}")
    elif kind == 1:
        out.append(f"1 {g.randrange(rows)} {g.randrange(cols - 1)} {g.randint(0, hi)}")
    else:
        out.append(f"2 {g.randrange(rows - 1)} {g.randrange(cols)} {g.randint(0, hi)}")
print("\n".join(out))
