import sys, random
# "0 S", then adds (cells stay in 0..=32767) and rectangle sums, then "3"; at most 60002 commands.
mx = sys.argv[1] == "--max"
r = random.Random(1 if mx else int(sys.argv[1]))
S = 1024 if mx else r.randint(1, max(1, min(int(sys.argv[2]), 8)))
q = 60000 if mx else r.randint(1, 20)
cell = {}
out = [f"0 {S}"]
for _ in range(q):
    if r.random() < 0.5:
        x, y = r.randrange(S), r.randrange(S)
        v = cell.get((x, y), 0)
        a = r.randint(-v, 32767 - v)
        cell[(x, y)] = v + a
        out.append(f"1 {x} {y} {a}")
    else:
        l, rr = sorted((r.randrange(S), r.randrange(S)))
        b, t = sorted((r.randrange(S), r.randrange(S)))
        out.append(f"2 {l} {b} {rr} {t}")
out.append("3")
print("\n".join(out))
