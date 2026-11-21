#!/usr/bin/env python3
import random
import sys


def build_expr(rng, l, r, force_op=None):
    if l == r:
        return str(l)
    op = force_op if force_op is not None else rng.choice(["min", "max"])
    # Split [l..r] into k >= 2 contiguous parts
    max_k = min(r - l + 1, 4)
    k = rng.randint(2, max_k)
    cuts = sorted(rng.sample(range(l, r), k - 1))
    bounds = [l - 1] + cuts + [r]
    next_op = "max" if op == "min" else "min"
    parts = [build_expr(rng, bounds[i] + 1, bounds[i + 1], next_op) for i in range(k)]
    return f"{op} ( " + " , ".join(parts) + " )"


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)
    budget = max(2, min(size, 400))
    cases = []
    while budget >= 2 and len(cases) < 5:
        n = rng.randint(2, min(budget, max(2, budget // 2 + 2)))
        if not cases and rng.random() < 0.4:
            n = budget
        cases.append(n)
        budget -= n
    print(len(cases))
    for n in cases:
        print(n)
        print(build_expr(rng, 1, n))
        q = rng.randint(2, 6)
        print(q)
        for _ in range(q):
            print(*[rng.randint(1, 10**9) for _ in range(n)])


if __name__ == "__main__":
    main()
