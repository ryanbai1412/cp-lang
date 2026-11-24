#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n = 2e5, values 1..3 (counts wrap the modulus many times)
    rng = random.Random(1)
    n = 200_000
    print(1)
    print(n)
    print(*[rng.randint(1, 3) for _ in range(n)])
    sys.exit()


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)
    t = rng.randint(1, max(1, min(6, size)))
    print(t)
    for _ in range(t):
        n = rng.randint(3, max(3, size))
        mode = rng.randrange(4)
        if mode == 0:
            a = [rng.randint(1, 3) for _ in range(n)]
        elif mode == 1:
            a = [1] * (n // 3) + [2] * (n // 3) + [3] * (n - 2 * (n // 3))
        elif mode == 2:
            a = [rng.choice([1, 2]) for _ in range(n)]
        else:
            a = [rng.choice([2, 3]) for _ in range(n)]
        print(n)
        print(*a)


if __name__ == "__main__":
    main()
