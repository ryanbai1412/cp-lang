#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n m = 2e5: n = 5e4 days of m = 4 drinks, profits near 1e9 (totals near 2e14)
    rng = random.Random(1)
    n, m = 50_000, 4
    print(1)
    print(n, m)
    print("\n".join(" ".join(str(rng.randint(-10**8, 10**9)) for _ in range(m)) for _ in range(n)))
    sys.exit()


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)
    if size >= 10000:
        n = rng.choice([1, 2, 1000, 66666])
        m = 200000 // n
        print(1)
        print(n, m)
        for _ in range(n):
            print(*[rng.randint(-10**9, 10**9) for _ in range(m)])
        return
    t = rng.randint(1, 5)
    print(t)
    for _ in range(t):
        n = rng.randint(1, 6)
        m = rng.randint(3, max(3, min(size, 8)))
        hi = rng.choice([3, 10, 10**9])
        print(n, m)
        for _ in range(n):
            print(*[rng.randint(-hi, hi) for _ in range(m)])


main()
