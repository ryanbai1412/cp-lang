"""NOI 2005 P2 (Sequence Maintenance): `N M`, N numbers in [-1000, 1000], then M <= 20000 ops
INSERT posi tot c..., DELETE / MAKE-SAME (c) / REVERSE / GET-SUM posi tot, MAX-SUM. Every op stays
inside the sequence, the sequence is never empty, at most 5e5 numbers are alive and at most 4e6
are inserted in all (the initial N included).

`gen.py SEED SIZE`: N and M up to SIZE, tot up to about SIZE. `gen.py --max`: N = 5e5, M = 2e4,
4000 DELETE + INSERT pairs of 875 numbers (4e6 inserted, 5e5 alive throughout) between long
REVERSE, MAKE-SAME, GET-SUM and MAX-SUM ranges.
"""
import random
import sys

if sys.argv[1:] == ["--max"]:
    r = random.Random(1)
    n, m, chunk, pairs = 500_000, 20_000, 875, 4000
    out = [f"{n} {m}", " ".join(str(r.randint(-1000, 1000)) for _ in range(n))]
    kinds = ["P"] * pairs + ["REVERSE", "MAKE-SAME", "GET-SUM", "MAX-SUM"] * ((m - 2 * pairs) // 4)
    r.shuffle(kinds)
    for k in kinds:
        if k == "P":
            out.append(f"DELETE {r.randint(1, n - chunk + 1)} {chunk}")
            vals = " ".join(str(r.randint(-1000, 1000)) for _ in range(chunk))
            out.append(f"INSERT {r.randint(0, n - chunk)} {chunk} {vals}")
        elif k == "MAX-SUM":
            out.append(k)
        else:
            tot = r.randint(n // 2, n)
            posi = r.randint(1, n - tot + 1)
            out.append(f"{k} {posi} {tot}" + (f" {r.randint(-1000, 1000)}" if k == "MAKE-SAME" else ""))
    print("\n".join(out))
    sys.exit()

r = random.Random(int(sys.argv[1]))
size = max(1, int(sys.argv[2]))
hi = r.choice([3, 1000])
val = lambda: r.randint(-hi, hi)
n, m = r.randint(1, size), r.randint(1, size)
alive, inserted = n, n
out = [f"{n} {m}", " ".join(str(val()) for _ in range(n))]
for _ in range(m):
    k = r.choice(["INSERT", "DELETE", "MAKE-SAME", "REVERSE", "GET-SUM", "MAX-SUM"])
    if k == "INSERT":
        tot = min(r.randint(1, size), 500_000 - alive, 4_000_000 - inserted)
        if tot < 1:
            k = "MAX-SUM"
        else:
            out.append(f"INSERT {r.randint(0, alive)} {tot} " + " ".join(str(val()) for _ in range(tot)))
            alive += tot
            inserted += tot
            continue
    if k == "DELETE" and alive == 1:
        k = "REVERSE"
    if k == "MAX-SUM":
        out.append(k)
        continue
    tot = r.randint(0 if k == "GET-SUM" else 1, alive - (k == "DELETE"))
    posi = r.randint(1, alive - tot + 1) if tot else r.randint(1, alive)
    out.append(f"{k} {posi} {tot}" + (f" {val()}" if k == "MAKE-SAME" else ""))
    if k == "DELETE":
        alive -= tot
print("\n".join(out))
