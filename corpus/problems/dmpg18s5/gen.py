import sys, random
# statement: 1 <= N, Q <= 2e5, values and x in 1..=2e5; "1 l r x" counts, "2 u v" sets.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 200000
n = 200000 if big else r.randint(1, max(1, min(S, 1200)))
q = 200000 if big else r.randint(1, 12)
top = 200000 if big else r.choice([12, 200000])
rich = [166320, 196560, 110880, 180180, 120120]
val = lambda: r.choice(rich) if big else r.randint(1, top)
print(n, q)
print(*(val() for _ in range(n)))
out = []
for _ in range(q):
    if r.random() < 0.5:
        l = r.randint(1, n); hi = r.randint(l, n)
        if big: l, hi = r.randint(1, 400), r.randint(n - 400, n)
        out.append(f"1 {l} {hi} {r.randint(1, 12) if big else r.randint(1, top)}")
    else:
        out.append(f"2 {r.randint(1, n)} {val()}")
print("\n".join(out))
