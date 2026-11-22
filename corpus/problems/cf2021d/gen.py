#!/usr/bin/env python3
import random
import sys


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
