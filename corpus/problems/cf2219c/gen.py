#!/usr/bin/env python3
"""CF 2219C: trees, n and sum n <= 2e5, at least one red node."""
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n: a 1e5 path from the only red node into a 1e5 star (depth plus a wide sort)
    r = random.Random(1)
    n, half = 200000, 100000
    es = [(i, i + 1) for i in range(1, half)] + [(half, i) for i in range(half + 1, n + 1)]
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    s = ["0"] * n
    s[perm[0] - 1] = "1"
    out = ["1", str(n), "".join(s)] + [f"{perm[u - 1]} {perm[v - 1]}" for u, v in es]
    print("\n".join(out))
    sys.exit()

r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])


def tree(n, shape):
    for i in range(2, n + 1):
        p = {"path": i - 1, "star": 1, "cat": max(1, i - 1 - (i % 2) * (i > 2)),
             "rand": r.randint(1, i - 1)}[shape]
        yield p, i


def case(n, shape, red):
    s = [("1" if r.random() < red else "0") for _ in range(n)]
    if "1" not in s:
        s[r.randrange(n)] = "1"
    return n, "".join(s), list(tree(n, shape))


if size >= 1000:
    mode = int(sys.argv[1]) % 5
    N = 200000
    tests = [[case(N, "path", 0)], [case(N, "star", 0)], [case(N, "rand", 0.1)],
             [case(N, "cat", 0.02)], [case(1, "rand", 1) for _ in range(1000)]][mode]
else:
    tests = [case(r.randint(1, 8 if size == 0 else 40), r.choice(["path", "star", "rand", "cat"]),
                  r.choice([0, 0.2, 0.5])) for _ in range(r.randint(1, 3))]
print(len(tests))
for n, s, es in tests:
    print(n)
    print(s)
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    for u, v in es:
        print(perm[u - 1], perm[v - 1])
