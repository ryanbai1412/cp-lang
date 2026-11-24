import sys, random
# seed size: n <= size + 1 events (a random permutation, sometimes nearly sorted), m <= 2 size
# rectangles (sometimes full height, c1 = 1 and c2 = n); --max: n = 1e5, m = 2e5 random rectangles
# over a random permutation, each wide and tall so it holds many pairs
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 100000 if mx else r.randint(1, min(100000, S + 1))
m = 200000 if mx else r.randint(1, min(200000, 2 * S))
p = list(range(1, n + 1))
kind = "rand" if mx else r.choice(["rand", "rand", "sorted", "full"])
if kind == "sorted":
    for _ in range(r.randint(0, 3)):
        i, j = r.randrange(n), r.randrange(n); p[i], p[j] = p[j], p[i]
else:
    r.shuffle(p)
out = [f"{n} {m}", " ".join(map(str, p))]
for _ in range(m):
    a, b = sorted((r.randint(1, n), r.randint(1, n)))
    c, d = (1, n) if kind == "full" else sorted((r.randint(1, n), r.randint(1, n)))
    if mx:
        a, b = r.randint(1, n // 10), r.randint(n - n // 10, n)
        c, d = r.randint(1, n // 10), r.randint(n - n // 10, n)
    out.append(f"{a} {b} {c} {d}")
print("\n".join(out))
