"""CF 2245 D2 inputs: `gen.py seed size`. size < 1e5 gives small random tests; size >= 1e5
gives a statement-max input picked by seed % 4: one test n = 2e5, m = 1e6 (YES); a complete
n = 1413 (m = n(n + 1) / 2); 1e4 tests of n = 20, m = 100; or n = 2e5, m = 1e6 with noise (NO)."""
import sys, random

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])


def pairs(n, m):
    if m * 3 >= n * (n + 1) // 2:
        allp = [(i, j) for i in range(1, n + 1) for j in range(i, n + 1)]
        return r.sample(allp, m)
    seen = set()
    while len(seen) < m:
        i, j = r.randint(1, n), r.randint(1, n)
        seen.add((min(i, j), max(i, j)))
    return list(seen)


def case(n, m, noise):
    a = r.sample(range(1, n + 1), n)
    a = [x if r.random() < 0.5 else -x for x in a]
    out = [f"{n} {m}"]
    for i, j in pairs(n, m):
        o = 1 if a[i - 1] + a[j - 1] >= 0 else 2
        if r.random() < noise:
            o = 3 - o
        out.append(f"{o} {i} {j}")
    return out


tests = []
if S < 100000:
    for _ in range(6):
        n = r.randint(1, 3 * S)
        tests.append(case(n, r.randint(0, min(4 * S, n * (n + 1) // 2)), r.choice([0, 0.02, 0.3])))
else:
    mode = int(sys.argv[1]) % 4
    if mode == 0:
        tests.append(case(200000, 1000000, 0))
    elif mode == 1:
        tests.append(case(1413, 1413 * 1414 // 2, 0))
    elif mode == 2:
        tests = [case(20, 100, r.choice([0, 0, 0.01])) for _ in range(10000)]
    else:
        tests.append(case(200000, 1000000, 1e-6))
print(len(tests))
print("\n".join(l for t in tests for l in t))
