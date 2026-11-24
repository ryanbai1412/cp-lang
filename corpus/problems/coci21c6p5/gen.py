import sys, random
# statement: 1 <= t <= 1e5 tests, each 1 <= n with sum n <= 1e5, a bracket string of length 2n and
# n pairs a < b covering 1..2n once.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 100000
if big:
    ns = [100000]
else:
    total = r.randint(1, max(1, min(S, 100000)))
    ns = []
    while total > 0:
        ns.append(r.randint(1, total)); total -= ns[-1]
print(len(ns))
for n in ns:
    # a balanced choice exists about half the time: plant one, then fill the partners at random
    chosen = []
    depth = 0
    for i in range(n):
        if depth == 0 or (depth < n - i and r.random() < 0.5):
            chosen.append("("); depth += 1
        else:
            chosen.append(")"); depth -= 1
    pos = list(range(1, 2 * n + 1))
    r.shuffle(pos)
    z = [""] * (2 * n + 1)
    pairs = []
    keep = sorted(pos[:n])
    for i, p in enumerate(keep):
        z[p] = chosen[i] if big or r.random() < 0.7 else r.choice("()")
    other = pos[n:]
    for i, p in enumerate(keep):
        q = other[i]
        z[q] = r.choice("()")
        pairs.append((min(p, q), max(p, q)))
    r.shuffle(pairs)
    print(n)
    print("".join(z[1:]))
    print("\n".join(f"{a} {b}" for a, b in pairs))
