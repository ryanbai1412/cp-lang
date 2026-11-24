#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n = 5e5 with 70% of the entries blank
    rng = random.Random(1)
    n = 500_000
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    print(1)
    print(n)
    print(*[0 if rng.random() < 0.7 else x for x in perm])
    sys.exit()


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)

    if size >= 10000:
        n = min(size, 500000)
        print(1)
        perm = list(range(1, n + 1))
        rng.shuffle(perm)
        a = [0 if rng.random() < 0.7 else x for x in perm]
        print(n)
        print(*a)
        return

    t = rng.randint(1, 5)
    print(t)
    for _ in range(t):
        n = rng.randint(1, max(1, min(size, 60)))
        perm = list(range(1, n + 1))
        rng.shuffle(perm)
        mode = rng.randint(0, 4)
        if mode == 0:
            a = [0] * n
        elif mode == 1:
            a = perm
        elif mode == 2:
            a = [0 if rng.random() < 0.8 else x for x in perm]
        elif mode == 3:
            a = [0 if rng.random() < 0.2 else x for x in perm]
        else:
            a = [0 if rng.random() < 0.5 else x for x in perm]
        print(n)
        print(*a)


if __name__ == "__main__":
    main()
