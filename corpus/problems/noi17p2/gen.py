"""noi17p2 generator: `n m`, n worm lengths in 1..=6, then m ops: `1 i j` joins the queue ending
at i to the one starting at j, `2 i` splits after i (at most 1000 splits), `3 s k` asks with a
digit string s (k <= |s|, k <= 50, sum |s| <= 1e7). n <= 2e5, m <= 5e5. Asks mostly read a run
of an existing queue so their counts are nonzero; mode picks few lengths, long chains, or churn."""
import random
import sys

if sys.argv[1] == "--max":
    # one chain of n = 2e5 worms, 1000 split/rejoin pairs, then asks of 33 digits read off the
    # chain until m = 5e5 and sum |s| is just under 1e7
    rng, n, m = random.Random(0), 200_000, 500_000
    size_of = [0] + [rng.randint(1, 6) for _ in range(n)]
    ops = [f"1 {i} {i + 1}" for i in range(1, n)]
    for _ in range(1000):
        i = rng.randint(1, n - 1)
        ops += [f"2 {i}", f"1 {i} {i + 1}"]
    while len(ops) < m:
        a = rng.randint(1, n - 32)
        ops.append(f"3 {''.join(str(size_of[w]) for w in range(a, a + 33))} {rng.randint(1, 33)}")
    print(n, m)
    print(*size_of[1:])
    print("\n".join(ops))
    sys.exit()

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 30
rng = random.Random(seed)
n = rng.randint(1, min(200_000, max(1, size)))
m = rng.randint(1, min(500_000, max(1, 3 * size)))
mode = rng.randrange(3)
top = rng.choice([1, 2, 6]) if mode == 0 else 6
size_of = [0] + [rng.randint(1, top) for _ in range(n)]
nxt, prv = [0] * (n + 1), [0] * (n + 1)
splits, budget, ops = 0, 10 * size + 100, []
for _ in range(m):
    tails = [w for w in range(1, n + 1) if not nxt[w]]
    links = [w for w in range(1, n + 1) if nxt[w]]
    weights = [5, 1, 3] if mode == 1 else [3, 3, 3] if mode == 2 else [2, 1, 3]
    kind = rng.choices([1, 2, 3], weights)[0]
    if kind == 1 and len(tails) < 2:
        kind = 3
    if kind == 2 and (not links or splits >= 1000):
        kind = 3
    if kind == 1:
        i = rng.choice(tails)
        head = i
        while prv[head]:
            head = prv[head]
        heads = [w for w in range(1, n + 1) if not prv[w] and w != head]
        j = rng.choice(heads)
        nxt[i], prv[j] = j, i
        ops.append(f"1 {i} {j}")
    elif kind == 2:
        i = rng.choice(links)
        j = nxt[i]
        nxt[i], prv[j] = 0, 0
        splits += 1
        ops.append(f"2 {i}")
    else:
        if rng.random() < 0.7:
            w = rng.randint(1, n)
            run = []
            while w and len(run) < 60:
                run.append(str(size_of[w]))
                w = nxt[w]
            s = "".join(run)
            if rng.random() < 0.3:
                s += "".join(str(rng.randint(1, top)) for _ in range(rng.randint(0, 5)))
        else:
            s = "".join(str(rng.randint(1, top)) for _ in range(rng.randint(1, 60)))
        s = s[: max(1, min(len(s), budget))]
        budget = max(1, budget - len(s))
        k = rng.randint(1, min(50, len(s)))
        ops.append(f"3 {s} {k}")
print(n, len(ops))
print(*size_of[1:])
print("\n".join(ops))
