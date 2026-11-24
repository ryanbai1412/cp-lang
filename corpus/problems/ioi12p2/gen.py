"""ioi12p2 (Parachute Rings) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N C" (1 <= N <= 1e6, 0 <= C <= 1e6 calls), then C calls: "u v" for Link(u, v) (rings 0..N-1,
distinct, not yet linked) or "-1 -1" for CountCritical().
--max: N = C = 1e6: three arms grow from ring 0 so it has three links and four candidates stay
alive, other rings form chains, and 4e5 CountCritical calls are spread among 6e5 links.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
calls = []
if MAX:
    n, links = 1_000_000, 600_000
    tails = [0, 0, 0]
    fresh = iter(range(1, n))
    for t in range(links):
        if t < 400_000:
            arm = t % 3 if t < 3 else rng.randrange(3)
            v = next(fresh)
            calls.append((tails[arm], v))
            tails[arm] = v
        else:
            a = next(fresh) if t % 2 == 0 else calls[-1][1]
            calls.append((a, next(fresh)) if t % 2 == 0 else (a, next(fresh)))
    out = []
    asks = 400_000
    for t, c in enumerate(calls):
        out.append(c)
        if t % 3 != 0 and asks:
            out.append((-1, -1))
            asks -= 1
    out += [(-1, -1)] * asks
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(1, min(1_000_000, size))
    linked = set()
    out = []
    for _ in range(rng.randint(0, min(1_000_000, 3 * size))):
        if n >= 2 and rng.random() < 0.6:
            u, v = rng.sample(range(n), 2)
            if (min(u, v), max(u, v)) not in linked:
                linked.add((min(u, v), max(u, v)))
                out.append((u, v))
                continue
        out.append((-1, -1))
print(n, len(out))
print("\n".join(f"{u} {v}" for u, v in out))
