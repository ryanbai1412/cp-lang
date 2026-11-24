#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n = 5000, about half the entries missing: the full O(n^2) sweep
    rng = random.Random(1)
    n = 5000
    perm = list(range(n))
    rng.shuffle(perm)
    print(1)
    print(n)
    print(*[-1 if rng.random() < 0.5 else x for x in perm])
    sys.exit()


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)

    if size >= 1000:
        n = min(size, 5000)
        print(1)
        perm = list(range(n))
        rng.shuffle(perm)
        miss_rate = rng.choice([0.3, 0.5, 0.7])
        a = [-1 if rng.random() < miss_rate else x for x in perm]
        print(n)
        print(*a)
        return

    t = rng.randint(1, 5)
    print(t)
    for _ in range(t):
        n = rng.randint(1, max(1, min(size, 40)))
        perm = list(range(n))
        rng.shuffle(perm)
        mode = rng.randint(0, 4)
        if mode == 0:
            a = [-1] * n
        elif mode == 1:
            a = perm
        elif mode == 2:
            a = [-1 if rng.random() < 0.8 else x for x in perm]
        elif mode == 3:
            a = [-1 if rng.random() < 0.2 else x for x in perm]
        else:
            a = [-1 if rng.random() < 0.5 else x for x in perm]
        print(n)
        print(*a)


if __name__ == "__main__":
    main()
