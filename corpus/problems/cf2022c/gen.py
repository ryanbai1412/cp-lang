#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n = 1e5 (n = 99999, the largest multiple of 3)
    rng = random.Random(1)
    n = 99999
    print(1)
    print(n)
    for _ in range(2):
        print("".join(rng.choice("AJ") for _ in range(n)))
    sys.exit()


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)
    if size >= 10000:
        n = 99999
        print(1)
        print(n)
        for _ in range(2):
            print("".join(rng.choice("AJ") for _ in range(n)))
        return
    t = rng.randint(1, 6)
    print(t)
    for _ in range(t):
        n = 3 * rng.randint(1, max(1, size))
        p = rng.random()
        print(n)
        for _ in range(2):
            print("".join("A" if rng.random() < p else "J" for _ in range(n)))


main()
