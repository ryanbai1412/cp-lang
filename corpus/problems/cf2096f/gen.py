import sys, random
if sys.argv[1] == "--max":
    # one test with n = m = q = 2e5 (all three sums): short "no" runs and longer "some" intervals,
    # so long windows fit and both pointers sweep the whole statement list
    rng = random.Random(1)
    n = m = q = 200_000
    out = ["1", f"{n} {m}"]
    for _ in range(m):
        t = rng.random() < 0.5
        w = rng.randint(1, 2000) if t else rng.randint(1, 30)
        a = rng.randint(1, n - w + 1)
        out.append(f"{int(t)} {a} {a + w - 1}")
    out.append(str(q))
    for _ in range(q):
        l = rng.randint(1, m); out.append(f"{l} {rng.randint(l, m)}")
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 2 + 3)
    m = rng.randint(1, size * 3 + 3)
    print(n, m)
    for _ in range(m):
        t = rng.randint(0, 1)
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        print(t, l, r)
    q = rng.randint(1, size * 2 + 2)
    print(q)
    for _ in range(q):
        l = rng.randint(1, m)
        r = rng.randint(l, m)
        print(l, r)
