#!/usr/bin/env python3
import random
import sys


def make_case(rng, n):
    a = list(range(1, n + 1))
    rng.shuffle(a)
    if rng.random() < 0.7:
        # Generate a valid reachable b by doing random valid operations, then mask some entries.
        b = a[:]
        ops = rng.randint(0, n)
        for _ in range(ops):
            l = rng.randint(0, n - 2)
            r = rng.randint(l + 1, min(n - 1, l + rng.randint(1, max(1, n // 4))))
            m = min(b[l : r + 1])
            pos = b.index(m, l, r + 1)
            if pos > l:
                b[l : pos + 1] = [b[pos]] + b[l:pos]
        keep = rng.choice([0.1, 0.3, 0.5, 0.8, 1.0])
        c = [x if rng.random() < keep else 0 for x in b]
    else:
        # Random partial permutation (often impossible).
        b = list(range(1, n + 1))
        rng.shuffle(b)
        keep = rng.choice([0.2, 0.5, 0.8])
        c = [x if rng.random() < keep else 0 for x in b]
    return a, c


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)

    if size >= 10000:
        n = min(size, 500000)
        print(1)
        a = list(range(1, n + 1))
        rng.shuffle(a)
        # Keep a valid permutation b = a with 40% fixed so the greedy runs to completion.
        c = [x if rng.random() < 0.4 else 0 for x in a]
        print(n)
        print(*a)
        print(*c)
        return

    t = rng.randint(1, 6)
    print(t)
    for _ in range(t):
        n = rng.randint(2, max(2, min(size, 50)))
        a, c = make_case(rng, n)
        print(n)
        print(*a)
        print(*c)


if __name__ == "__main__":
    main()
