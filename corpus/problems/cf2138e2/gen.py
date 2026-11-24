#!/usr/bin/env python3
"""Generator for cf2138e2 (Determinant Construction, Hard Version)."""
import random
import sys


def main():
    if sys.argv[1:] == ["--max"]:
        seed, size = 2138, "max"
    else:
        seed = int(sys.argv[1])
        size = sys.argv[2] if len(sys.argv) > 2 else "small"
    rng = random.Random(seed)
    if size == "small":
        t = rng.randint(1, 5)
        xs = [rng.randint(0, 500) for _ in range(t)]
    elif size == "medium":
        t = rng.randint(5, 15)
        xs = [rng.randint(0, 200_000) for _ in range(t)]
    elif size == "max":
        t = 500
        fixed = [0, 1, 2, 3, 4, 8_418_060, 9_999_991, 10_000_000]
        xs = fixed + [rng.randint(2, 10_000_000) for _ in range(t - len(fixed))]
    else:
        n = int(size)
        t = max(1, min(5, n))
        xs = [rng.randint(0, 10 ** min(7, n)) for _ in range(t)]
    print(t)
    for x in xs:
        print(x)


if __name__ == "__main__":
    main()
