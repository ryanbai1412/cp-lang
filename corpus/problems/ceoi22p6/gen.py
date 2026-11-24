"""gen.py SEED SIZE | --max: CEOI 2022 Parking. N colours with two vehicles each in M >= N spaces of
capacity two, 1 <= N <= M <= 200000; a space is empty, full, or holds a bottom vehicle only. Some
cases have every space empty or full, some keep a few pairs already parked together. --max is
M = 200000 with N = 199000 (2000 free slots)."""
import random, sys

MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
if MAX:
    m, n = 200_000, 199_000
else:
    n = r.randint(1, 1 + S)
    m = r.randint(n, n + r.choice([0, 1, 2, max(1, S // 2)]))
cars = [c for c in range(1, n + 1) for _ in range(2)]
r.shuffle(cars)
full_only = not MAX and r.random() < 0.3
sp = [[] for _ in range(m)]
if full_only:
    for i in range(0, 2 * n, 2):
        sp[i // 2] = cars[i:i + 2]
else:
    free = 2 * m - 2 * n
    singles = min(free, r.randint(0, min(free, m)) if not MAX else 2000)
    k = 0
    idx = list(range(m))
    r.shuffle(idx)
    for i in idx:
        if k == 2 * n:
            break
        take = 1 if singles > 0 and (2 * n - k) > 0 and (2 * n - k) <= 2 * len([0]) * m else 2
        if singles > 0 and r.random() < singles / max(1, m):
            take = 1
            singles -= 1
        take = min(take, 2 * n - k)
        sp[i] = cars[k:k + take]
        k += take
    while k < 2 * n:
        for i in idx:
            if k < 2 * n and len(sp[i]) < 2:
                sp[i].append(cars[k])
                k += 1
r.shuffle(sp)
out = [f"{n} {m}"] + [f"{s[0] if s else 0} {s[1] if len(s) > 1 else 0}" for s in sp]
sys.stdout.write("\n".join(out) + "\n")
