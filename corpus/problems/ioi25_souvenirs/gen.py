import sys, random

P0 = 10**15


def falling(r, n, mode):
    """n strictly falling prices in 1..=1e15, shaped like one of the subtasks."""
    if mode == 0:  # consecutive: P[i] = N - i, shifted up
        base = r.choice([1, 1, r.randint(1, P0 - n)])
        return [base + n - 1 - i for i in range(n)]
    if mode == 1:  # gaps of 1 or 2
        p = [r.choice([1, 1, r.randint(1, 1000)])]
        for _ in range(n - 1):
            p.append(p[-1] + r.randint(1, 2))
        return p[::-1]
    if mode == 2:  # superincreasing: P[i] >= P[i+1] + P[i+2]
        p = [r.randint(1, 3)]
        for _ in range(n - 1):
            p.append(p[-1] + (p[-2] if len(p) > 1 else 0) + r.randint(1, 3))
        if p[-1] > P0:
            return None
        return p[::-1]
    if mode == 3:  # random, small values
        hi = r.choice([n, 2 * n, 10 * n, 1000])
        return sorted(r.sample(range(1, hi + 1), n), reverse=True)
    if mode == 4:  # near the top
        return sorted(r.sample(range(P0 - r.choice([n, 10 * n, 10**6]) + 1, P0 + 1), n), reverse=True)
    if mode == 5:  # each price a random fraction of the one before
        p = [r.randint(P0 // 2, P0)]
        for i in range(1, n):
            lo = n - i
            p.append(r.randint(max(lo, p[-1] // 2 - 5), p[-1] - 1) if p[-1] - 1 >= lo else None)
            if p[-1] is None:
                return None
        return p
    hi = r.choice([10**6, 10**12, P0])  # random over a wide range
    return sorted(r.sample(range(1, hi + 1), n), reverse=True)


def emit(n, p):
    assert 2 <= n <= 100 and len(p) == n and p[0] <= P0 and p[-1] >= 1
    assert all(a > b for a, b in zip(p, p[1:]))
    print(n)
    print(" ".join(map(str, p)))


if sys.argv[1:] == ["--max"]:
    # N = 100 with P[0] = 1e15 and each price about half the one before: the deepest
    # chains of leads, and every price wide.
    r = random.Random(2025)
    p = [P0]
    for i in range(1, 100):
        p.append(r.randint(max(100 - i, p[-1] // 2 - 5), p[-1] - 1))
    emit(100, p)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = min(100, r.randint(2, max(2, S)) if S < 100 else 100)
while True:
    p = falling(r, n, r.randrange(7))
    if p:
        break
emit(n, p)
