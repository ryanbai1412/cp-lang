import sys, random
# gen.py seed size: t <= 3 instances of n <= 4 size + 2 constraints over indices in a small range or
# up to 1e9. Each index gets a hidden class: equalities join indices of one class, inequalities
# split two classes, and some instances get one contradicting inequality. --max: t = 10 instances
# of n = 1e5 over indices up to 1e9 (long equality chains), every other one contradicted.
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
t = 10 if MAX else r.randint(1, 3)
out = [str(t)]
for k in range(t):
    n = 10**5 if MAX else r.randint(1, 4 * int(sys.argv[2]) + 2)
    top = 10**9 if MAX or r.random() < 0.5 else max(2, n)
    idx = [r.randint(1, top) for _ in range(min(n, top))]
    classes = r.randint(1, max(1, len(idx) // 20)) if MAX else r.randint(1, 4)
    cls = {}
    for i in idx: cls.setdefault(i, r.randrange(classes))
    by = {}
    for i in cls: by.setdefault(cls[i], []).append(i)
    rows = []
    for _ in range(n):
        i = r.choice(idx)
        if r.random() < 0.7 or classes == 1:
            rows.append((i, r.choice(by[cls[i]]), 1))
        else:
            c = r.choice([c for c in by if c != cls[i]] or [cls[i]])
            rows.append((i, r.choice(by[c]), 0 if c != cls[i] else 1))
    if (k % 2 if MAX else r.random() < 0.4):
        i = r.choice(idx); rows[r.randrange(n)] = (i, r.choice(by[cls[i]]), 0)
    out.append(str(n)); out += [f"{i} {j} {e}" for i, j, e in rows]
print("\n".join(out))
