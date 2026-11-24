import sys, random

if sys.argv[1:] == ["--max"]:
    # q = 3e5 queries with n over the whole 0..3e5 range (every block of the split gets used) and
    # k at the top of 0..2n. Those prefix sums have closed forms (the total minus the top one or
    # two coefficients); a few dozen random k at n near 3e5 are summed directly.
    M = 10**9 + 7
    r = random.Random(2159)
    a, b, c = (r.randint(M // 2, M - 1) for _ in range(3))
    q, nmax = 300_000, 300_000
    inv = [0, 1]
    for i in range(2, 2 * nmax + 1):
        inv.append((M - (M // i) * inv[M % i] % M) % M)
    ic = pow(c, M - 2, M)

    def direct(n, k):
        f0, f1, s = pow(c, n, M), 0, 0
        s = f0
        prev2, prev1 = 0, f0
        for z in range(1, k + 1):
            acc = prev1 * b % M * (n - z + 1)
            if z >= 2:
                acc += prev2 * a % M * (2 * n - z + 2)
            cur = acc % M * inv[z] % M * ic % M
            s += cur
            prev2, prev1 = prev1, cur
        return s % M

    def top(n, d):
        # sum of the coefficients of x^0..x^(2n - d), d in 0..=2
        tot = pow(a + b + c, n, M)
        if d >= 1 and n >= 1:
            tot -= pow(a, n, M)
        if d >= 2 and n >= 1:
            tot -= n * pow(a, n - 1, M) * b
        return tot % M

    direct_at = set(r.sample(range(q), 24))
    print(a, b, c)
    print(q)
    out, last = [], 0
    for i in range(q):
        if i in direct_at:
            n = r.randint(nmax - 1000, nmax)
            k = r.randint(n, 2 * n)
            ans = direct(n, k)
        else:
            n = nmax if i % 97 == 0 else r.randint(1, nmax)
            d = r.randint(0, 2)
            k = 2 * n - d
            ans = top(n, d)
        out.append(f"{n ^ last} {k ^ last}")
        last = ans
    print("\n".join(out))
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
M = 10**9 + 7
a, b, c = (r.choice([1, 2, M - 1, r.randint(1, M - 1)]) for _ in range(3))
q = r.randint(1, 3 * S)
nmax = r.choice([5, 50, 700 * S])
inv = [0, 1]
for i in range(2, 2 * nmax + 1):
    inv.append((M - (M // i) * inv[M % i] % M) % M)
ic = pow(c, M - 2, M)


def answer(n, k):
    f = [pow(c, n, M)]
    for z in range(1, k + 1):
        acc = f[z - 1] * b % M * (n - z + 1)
        if z >= 2:
            acc += f[z - 2] * a % M * (2 * n - z + 2)
        f.append(acc % M * inv[z] % M * ic % M)
    return sum(f) % M


print(a, b, c)
print(q)
last = 0
for _ in range(q):
    n = r.randint(0, nmax)
    k = r.choice([0, 2 * n, r.randint(0, 2 * n)])
    print(n ^ last, k ^ last)
    last = answer(n, k)
