"""sdoi17r2p2 (SDOI 2017 Apple Tree) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

Q <= 5 datasets, each "n k" then n lines "parent a v": 1 <= n <= 20000, 1 <= k <= 5e5,
n k <= 2.5e7, parent of node 1 is 0 and of node i a smaller node, 1 <= a <= 1e8,
1 <= v <= 100. Trees are random, height 2 (star), a path, or caterpillar-like; some
datasets have every a_i = 1 (a subtask) and some have huge a_i.
--max: five datasets at n k = 2.5e7: a random tree, a path, a star with all a_i = 1,
a broom with large a_i, and n = 50 with k = 5e5.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))


def parents(n, shape):
    if shape == "star":
        return [0] + [1] * (n - 1)
    if shape == "path":
        return list(range(n))
    if shape == "broom" and n >= 4:
        h = n // 2
        return list(range(h)) + [rng.randint(max(1, h - 5), h) for _ in range(n - h)]
    return [0] + [rng.randint(max(1, i - rng.choice([1, 3, i])), i - 1) for i in range(2, n + 1)]


def dataset(n, k, shape, amax):
    par = parents(n, shape)
    rows = [f"{par[i]} {rng.randint(1, amax)} {rng.randint(1, 100)}" for i in range(n)]
    return f"{n} {k}\n" + "\n".join(rows)


out = []
if MAX:
    out.append(dataset(20000, 1250, "random", 10**8))
    out.append(dataset(20000, 1250, "path", 3))
    out.append(dataset(20000, 1250, "star", 1))
    out.append(dataset(20000, 1250, "broom", 10**8))
    out.append(dataset(50, 500000, "random", 10**8))
else:
    size = max(1, int(sys.argv[2]))
    for _ in range(rng.randint(1, 5)):
        n = rng.randint(1, min(20000, size))
        k = rng.randint(1, min(500000, 25 * 10**6 // n, max(1, 2 * size)))
        amax = rng.choice([1, 3, max(1, size), 10**8])
        out.append(dataset(n, k, rng.choice(["random", "star", "path", "broom"]), amax))
print(len(out))
print("\n".join(out))
