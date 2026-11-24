import sys, random
if sys.argv[1] == "--max":
    # n = q = 1e5; inserts and reversals of long prefixes pile tags on deep treap paths,
    # deletes hit live ids, queries go everywhere
    rng = random.Random(1)
    n = q = 100_000
    out = [f"{n} {q}"]
    for i in range(q):
        a = rng.choices((1, 2, 3), (5, 4, 1))[0]
        b = rng.randint(1, q) if a == 3 else rng.randint(n // 2, n)
        out.append(f"{a} {b} {rng.randint(1, n)}")
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = rng.randint(2, size * 3 + 3)
q = rng.randint(2, size * 4 + 4)
print(n, q)
for _ in range(q):
    a = rng.randint(1, 3)
    b = rng.randint(1, q if a == 3 else n)
    c = rng.randint(1, n)
    print(a, b, c)
