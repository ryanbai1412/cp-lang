import sys, random

if sys.argv[1] == "--max":
    # n = Q = 2.5e5: a random a against a near-decreasing b (deep monotone stacks, many range
    # assignments per step), queries mostly wide
    rng = random.Random(1)
    n = q = 250_000
    a = list(range(1, n + 1)); rng.shuffle(a)
    b = list(range(n, 0, -1))
    for _ in range(2000):
        i, j = rng.randrange(n), rng.randrange(n); b[i], b[j] = b[j], b[i]
    out = [f"25 {n}", " ".join(map(str, a)), " ".join(map(str, b)), str(q)]
    for _ in range(q):
        l, rr = sorted((rng.randint(1, n // 10), rng.randint(n - n // 10, n))) if rng.random() < 0.5 else sorted((rng.randint(1, n), rng.randint(1, n)))
        out.append(f"{l} {rr}")
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: T n; permutations a, b of 1..n; Q queries 1 <= l <= r <= n; n, Q <= 2.5e5
n = r.randint(1, S) if S <= 1000 else S
q = r.randint(1, S) if S <= 1000 else S
def perm(mode):
    p = list(range(1, n + 1))
    if mode == 0: r.shuffle(p)
    elif mode == 1: p = p[::-1]
    elif mode == 2: pass
    else:
        for _ in range(min(n, 20)):
            i, j = sorted((r.randrange(n), r.randrange(n))); p[i:j + 1] = p[i:j + 1][::-1]
    return p
ma, mb = r.randrange(4), r.randrange(4)
wide = r.random() < 0.5
out = [f"{r.randint(0, 25)} {n}", " ".join(map(str, perm(ma))), " ".join(map(str, perm(mb))), str(q)]
for _ in range(q):
    if wide and r.random() < 0.8:
        l, rr = r.randint(1, min(n, 10)), r.randint(max(1, n - 10), n)
        l, rr = min(l, rr), max(l, rr)
    else:
        l, rr = sorted((r.randint(1, n), r.randint(1, n)))
    out.append(f"{l} {rr}")
print("\n".join(out))
