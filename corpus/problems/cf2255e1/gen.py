import sys, random
if sys.argv[1] == "--max":  # one case n = q = 5e5 (both sums), values +-1, a mix of all four ops
    rng = random.Random(0); n = q = 5 * 10**5; lim = 1
    out = ["1", f"{n} {q}", " ".join(str(rng.choice([-lim, lim, rng.randint(-lim, lim)])) for _ in range(n))]
    for _ in range(q):
        t = rng.choices([1, 2, 3, 4], [2, 2, 2, 3])[0]
        u, v = rng.randrange(2 ** 64), rng.randrange(2 ** 64)
        out.append(f"1 {u} {v} {rng.randint(-lim, lim)}" if t == 1 else f"4 {u}" if t == 4 else f"{t} {u} {v}")
    print("\n".join(out)); sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
big = size > 100
tests = [(size, size)] if big else [(rng.randint(1, 2 * size), rng.randint(1, 4 * size)) for _ in range(rng.randint(1, 4))]
lim = 10 ** 9 if big or rng.random() < 0.5 else 5
def coord():
    return rng.randrange(2 ** 64) if rng.random() < 0.5 else rng.randint(0, 20)
print(len(tests))
for n, q in tests:
    print(n, q)
    print(*[rng.randint(-lim, lim) for _ in range(n)])
    for _ in range(q):
        t = rng.choices([1, 2, 3, 4], [2, 2, 2, 3])[0]
        if t == 1: print(1, coord(), coord(), rng.randint(-lim, lim))
        elif t == 4: print(4, coord())
        else: print(t, coord(), coord())
