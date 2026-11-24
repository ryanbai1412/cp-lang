#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n = 3e5, a_i up to 1e9, b over three values so runs keep merging
    rng = random.Random(1)
    n = 300_000
    out, prev = [], -1
    for _ in range(n):
        b = rng.choice([v for v in (0, 1, 2) if v != prev])
        out.append(f"{rng.randint(1, 10**9)} {b}")
        prev = b
    print(1)
    print(n)
    print("\n".join(out))
    sys.exit()


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)
    if size >= 10000:
        tests = [(300000, rng.choice([2, 3, 300000]), 10**9)]
    else:
        tests = [(rng.randint(1, 3 * size), rng.randint(1, 4), rng.choice([3, 10, 10**9]))
                 for _ in range(rng.randint(1, 5))]
    print(len(tests))
    for n, vals, hi in tests:
        print(n)
        prev = -1
        for _ in range(n):
            b = rng.choice([v for v in range(min(vals, n) + 1) if v != prev])
            print(rng.randint(1, hi), b)
            prev = b


main()
