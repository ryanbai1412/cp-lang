#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at n = q = 200000: a randomly grown Young diagram under random row and column
    # permutations, so the matrix stays monotone and every answer is YES
    def young(n, q, rng):
        """q cells of a randomly grown Young diagram, rows and columns permuted: monotone at every step."""
        row = [0] * (n + 2)
        row[0] = n
        addable, at = [1], {1: 0}
    
        def drop(r):
            i = at.pop(r)
            last = addable.pop()
            if last != r:
                addable[i] = last
                at[last] = i
    
        cells = []
        for _ in range(q):
            r = addable[rng.randrange(len(addable))]
            row[r] += 1
            cells.append((r, row[r]))
            if row[r] == row[r - 1] or row[r] == n:
                drop(r)
            if r < n and row[r + 1] == row[r] - 1 and r + 1 not in at:
                at[r + 1] = len(addable)
                addable.append(r + 1)
        pr, pc = list(range(1, n + 1)), list(range(1, n + 1))
        rng.shuffle(pr)
        rng.shuffle(pc)
        return [(pr[r - 1], pc[c - 1]) for r, c in cells]
    n = q = 200000
    g = random.Random(1)
    out = ["1", f"{n} {q}"] + [f"{r} {c}" for r, c in young(n, q, g)]
    print("\n".join(out))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 10)
    q = rng.randint(1, n)
    print(n, q)
    c0 = [0] * (n + 1)
    c1 = [0] * (n + 1)
    for _ in range(q):
        r0 = rng.choice([i for i in range(1, n + 1) if c0[i] < n - 1])
        r1 = rng.choice([i for i in range(1, n + 1) if c1[i] < n - 1])
        c0[r0] += 1; c1[r1] += 1
        print(r0, r1)
