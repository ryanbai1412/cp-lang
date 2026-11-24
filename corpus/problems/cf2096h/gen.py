import sys, random
if sys.argv[1] == "--max":
    # one test with n = 2e5 and m = 18 (sum 2^m = 2^18), wide random intervals
    rng = random.Random(1)
    n, m = 200_000, 18
    out = ["1", f"{n} {m}"]
    for _ in range(n):
        l = rng.randrange(1 << m); out.append(f"{l} {rng.randint(l, (1 << m) - 1)}")
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(1, size + 2)
    m = rng.randint(1, min(6, size + 1))
    lim = (1 << m) - 1
    print(n, m)
    for _ in range(n):
        l = rng.randint(0, lim)
        r = rng.randint(l, lim)
        print(l, r)
