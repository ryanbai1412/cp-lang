"""gen.py seed size: Mock CCC '22 J5. 1 <= N <= 5000, 1 <= T_i <= 1e4, 0 <= R_i < N.
size <= 20 is a small random case; larger sizes cycle through max-N shapes:
a dense DAG (sum R ~ N^2 / 4), a 5000-chain of T = 1e4, and a DAG mixed with loops."""
import random, sys

if sys.argv[1] == "--max":
    # N = 5000 in a shuffled order where each event needs every earlier one (R_i up to N - 1,
    # about N^2 / 2 prerequisites in all), times up to 1e4
    rng = random.Random(1)
    n = 5000
    order = list(range(n)); rng.shuffle(order)
    deps = [[] for _ in range(n)]
    for k, i in enumerate(order): deps[i] = order[:k]
    out = [str(n)]
    for i in range(n):
        out.append(" ".join(map(str, [rng.randint(1, 10**4), len(deps[i])] + [d + 1 for d in deps[i]])))
    print("\n".join(out))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size <= 20:
    n = rng.randint(1, size + 2)
    shape = rng.choice(["dag", "loops"])
else:
    n = 5000
    shape = ["dense", "chain", "loops"][seed % 3]
deps = [[] for _ in range(n)]
order = list(range(n)); rng.shuffle(order)
for k, i in enumerate(order):
    if shape == "chain":
        deps[i] = [order[k - 1]] if k else []
    elif shape == "dense":
        deps[i] = rng.sample(order[:k], rng.randint(0, k))
    else:
        r = rng.randint(0, min(n - 1, 3 if n > 20 else n - 1))
        pool = order[:k] if (shape == "dag" or rng.random() < 0.9) else list(range(n))
        deps[i] = rng.sample(pool, min(r, len(pool)))
tmax = 10 ** 4 if size > 20 else rng.choice([5, 10 ** 4])
out = [str(n)]
for i in range(n):
    t = tmax if shape == "chain" else rng.randint(1, tmax)
    out.append(" ".join(map(str, [t, len(deps[i])] + [d + 1 for d in deps[i]])))
print("\n".join(out))
