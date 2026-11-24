import sys, random
if len(sys.argv) > 1 and sys.argv[1] == "--max":
    # one n = 1e5 test (sum n) splitting a permutation of 1..=2n, every value distinct, so the
    # binary search runs its full log(2n) rounds over the whole array
    sys.argv[1:] = ["0", str(10 ** 5)]

# gen.py seed size: t tests, rows a and b of n values in 1..=2n (repeats allowed), sum n <= 1e5.
# No arguments, or size >= 1e5, uses the full limit; the seed picks one test of n = 1e5
# (a permutation split, random repeats, or few distinct values) or t = 1e4 tests of n = 10.
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
S = int(sys.argv[2]) if len(sys.argv) > 2 else 10 ** 5
r = random.Random(seed)
def rows(n, kind):
    if kind == 0:
        p = list(range(1, 2 * n + 1)); r.shuffle(p); return p[:n], p[n:]
    if kind == 1:
        return [r.randint(1, 2 * n) for _ in range(n)], [r.randint(1, 2 * n) for _ in range(n)]
    vals = [r.randint(1, 2 * n) for _ in range(3)]
    return [r.choice(vals) for _ in range(n)], [r.choice(vals) for _ in range(n)]
if S >= 10 ** 5:
    kind = seed % 4
    tests = [(10 ** 5, kind)] if kind < 3 else [(10, r.randint(0, 2)) for _ in range(10 ** 4)]
else:
    tests = [(r.randint(1, 2 * S + 3), r.randint(0, 2)) for _ in range(5)]
print(len(tests))
for n, kind in tests:
    a, b = rows(n, kind)
    print(n); print(*a); print(*b)
