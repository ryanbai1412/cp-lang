#!/usr/bin/env python3
"""CF 2152H1: trees, 2 <= n, sum n <= 2.5e5, w <= 1e9, q <= 10 queries l <= 1e9."""
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
M = 10 ** 9


def case(n, shape, wmax, lmax, q):
    out = [str(n)]
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    for v in range(2, n + 1):
        u = {"path": v - 1, "star": 1, "rand": rng.randint(1, v - 1),
             "broom": v - 1 if v <= n // 2 else 1}[shape]
        out.append(f"{perm[u - 1]} {perm[v - 1]} {rng.randint(max(1, wmax - 100), wmax)}")
    out.append(str(q))
    out += [str(rng.randint(max(1, lmax - 100), lmax)) for _ in range(q)]
    return out


if size >= 1000:
    N = 250000
    mode = int(sys.argv[1]) % 5
    tests = [[case(N, "path", M, M, 10)], [case(N, "star", M, M, 10)],
             [case(N, "rand", M, M, 10)], [case(N, "broom", 3, M, 10)],
             [case(2, "rand", M, M, 10) for _ in range(10000)]][mode]
else:
    tests = [case(rng.randint(2, max(3, size * 4)), rng.choice(["path", "star", "rand", "broom"]),
                  rng.choice([20, M]), rng.choice([40, M]), rng.randint(1, 10))
             for _ in range(rng.randint(1, 3))]
print(len(tests))
for t in tests:
    print("\n".join(t))
