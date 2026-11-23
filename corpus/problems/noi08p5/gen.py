"""noi08p5 generator: `q m`, then ops in time order: `1 t c l r d` adds cloud c over
[l, r] moving d (it never starts moving out of [0, m]), `2 t l r` asks, `3 t c` removes."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
m = rng.randint(1, min(1000, max(1, size)))
q = rng.randint(1, max(1, 2 * size))
t, live, fresh, ops = 0, [], 1, []
for _ in range(q):
    t += rng.choice([0, 0, 1, rng.randint(0, 3 * m)])
    kind = rng.choice([1, 2, 3] if live else [1, 2])
    if kind == 1:
        l = rng.randint(0, m - 1)
        r = rng.randint(l + 1, m)
        d = 1 if l == 0 else -1 if r == m else rng.choice([-1, 1])
        ops.append(f"1 {t} {fresh} {l} {r} {d}")
        live.append(fresh)
        fresh += rng.randint(1, 3)
    elif kind == 2:
        l = rng.randint(0, m)
        ops.append(f"2 {t} {l} {rng.randint(l, m)}")
    else:
        ops.append(f"3 {t} {live.pop(rng.randrange(len(live)))}")
print(len(ops), m)
print("\n".join(ops))
