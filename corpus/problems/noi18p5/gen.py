import sys, random
# seed size: T <= 3 cases of n <= size + 1 nodes (parent a < child b: a path, a random tree or a
# broom), m <= 2 size plans (some x = y), c and v of a random magnitude; --max: T = 20 cases of
# n = 5e4, m = 1e5 (sums 1e6 and 2e6) cycling path, random tree and broom, c up to 1e9
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
T = 20 if mx else r.randint(1, 3)
out = [str(T)]
for t in range(T):
    n = 50000 if mx else r.randint(1, min(50000, S + 1))
    m = 100000 if mx else r.randint(0, min(100000, 2 * S))
    shape = t % 3 if mx else r.randrange(3)
    cap = 10**9 if mx else r.choice([0, 3, 10**9])
    out.append(str(n))
    for b in range(2, n + 1):
        a = b - 1 if shape == 0 or (shape == 2 and b <= n // 2) else r.randint(1, b - 1)
        out.append(f"{a} {b} {r.randint(0, cap)}")
    vcap = min(10**10 * n, max(1, cap) * r.choice([1, n]) * 2)
    out.append(str(m))
    for _ in range(m):
        x = r.randint(1, n); y = x if r.random() < 0.05 else r.randint(1, n)
        out.append(f"{x} {y} {r.randint(0, vcap)}")
print("\n".join(out))
