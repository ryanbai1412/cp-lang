"""noi08p5 generator: `n len`, then n events in nondecreasing time: `1 t c l r d` adds a cloud of
colour c over [l, r] (0 <= l <= r <= len) moving d in {-1, 1}, `2 t l r` asks, `3 t c` removes a
cloud in the sky. 0 <= t <= 2e9, 1 <= c <= 1e6, n <= 2e5, len <= 1000. Above size 1000 it emits
n = size events with len = 1000; mode picks wide clouds, short clouds, or many asks."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
big = size > 1000
m = 1000 if big else rng.randint(1, min(1000, max(1, size)))
q = size if big else rng.randint(1, max(1, 2 * size))
mode = rng.randrange(3)
end_t = 2_000_000_000
t, live, free, ops = rng.choice([0, end_t - 3 * q]), [], [], []
colours = list(range(1, 1_000_001)) if big else list(range(1, 2 * q + 2))
rng.shuffle(colours)
fresh = iter(colours)
for i in range(q):
    if rng.random() < 0.3:
        t = min(end_t, t + rng.choice([1, rng.randint(0, 3 * m), rng.randint(0, 10**7)]))
    if i == q - 1 and rng.random() < 0.5:
        t = end_t
    weights = [3, 2, 1] if mode == 0 else [2, 6, 1] if mode == 1 else [3, 3, 2]
    kind = rng.choices([1, 2, 3], weights)[0]
    if kind == 3 and not live:
        kind = 1
    if kind == 1:
        w = rng.randint(0, m) if mode != 1 else rng.randint(0, min(m, 3))
        l = rng.randint(0, m - w)
        c = free.pop(rng.randrange(len(free))) if free and rng.random() < 0.3 else next(fresh)
        ops.append(f"1 {t} {c} {l} {l + w} {rng.choice([-1, 1])}")
        live.append(c)
    elif kind == 2:
        l = rng.randint(0, m)
        r = rng.randint(l, m) if rng.random() < 0.7 else l
        ops.append(f"2 {t} {l} {r}")
    else:
        c = live.pop(rng.randrange(len(live)))
        free.append(c)
        ops.append(f"3 {t} {c}")
print(len(ops), m)
print("\n".join(ops))
