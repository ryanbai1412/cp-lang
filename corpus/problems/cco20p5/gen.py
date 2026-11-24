import sys, random

if sys.argv[1:] == ["--max"]:
    # Q = 5e5 events on [1, 1e6]: wide random intervals, nested ones around the centre and
    # repeated copies, 40% removals of a live copy (never emptying the collection).
    rng = random.Random(2020)
    q, C = 500_000, 1_000_000
    live, ops = [], []
    for _ in range(q):
        if len(live) > 1 and rng.random() < 0.4:
            i = rng.randrange(len(live)); live[i], live[-1] = live[-1], live[i]
            l, r = live.pop(); ops.append(f"R {l} {r}")
            continue
        k = rng.random()
        if live and k < 0.15: l, r = rng.choice(live)
        elif k < 0.5: m = C // 2; l, r = m - rng.randint(0, m - 1), m + rng.randint(1, m)
        else: l, r = sorted(rng.sample(range(1, C + 1), 2))
        live.append((l, r)); ops.append(f"A {l} {r}")
    print(q)
    print("\n".join(ops))
    sys.exit()
# seed size: Q = size events (capped at 5e5) adding or removing one copy of [l, r],
# 1 <= l < r <= C; removals name a live copy and never empty the collection.
# shapes: tiny coordinates (duplicates, shared ends), wide random, short intervals
# on a big line, nested around a centre, the disjoint subtask (r1 < l2 always)
rng = random.Random(int(sys.argv[1])); S = max(1, min(int(sys.argv[2]), 500000))
q = S if rng.random() < 0.4 else rng.randint(1, S)
kind = rng.randrange(5)
C = rng.choice([3, 6, 12, 50, 1000, 1000000]) if kind < 2 else 1000000
p_rm = rng.choice([0.2, 0.4, 0.5])
live, ops = [], []
used = bytearray(C + 2)

def fresh():
    if kind == 2:
        l = rng.randint(1, C - 1); return l, min(C, l + rng.randint(1, 3))
    if kind == 3:
        m = C // 2; return m - rng.randint(0, m - 1), m + rng.randint(1, m)
    if kind == 4:
        while True:
            l = rng.randint(1, C - 1); r = min(C, l + rng.randint(1, 20))
            if not any(used[l:r + 1]): return l, r
    l, r = sorted(rng.sample(range(1, C + 1), 2)); return l, r

for _ in range(q):
    if len(live) > 1 and rng.random() < p_rm:
        i = rng.randrange(len(live)); live[i], live[-1] = live[-1], live[i]
        l, r = live.pop(); ops.append(f"R {l} {r}")
        if kind == 4: used[l:r + 1] = bytes(r - l + 1)
    else:
        if kind != 4 and live and rng.random() < 0.15: l, r = rng.choice(live)
        else: l, r = fresh()
        if kind == 4: used[l:r + 1] = b"\1" * (r - l + 1)
        live.append((l, r)); ops.append(f"A {l} {r}")
print(q)
print('\n'.join(ops))
