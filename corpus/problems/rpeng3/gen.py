import sys, random
# seed size: N <= size (capped at 5e5), locations distinct in 1..=500001 and never home, 1 <= D <= U <= 10
# --max: N = 5e5 fairs filling every location but home, U = 10 > D = 1, days mixing one-fair days
# with crowded ones (every fair a SegTree fold and update, every crowded day a full sweep).
if sys.argv[1] == "--max":
    r, n = random.Random(0), 500000
    home = 250001
    locs = [x for x in range(1, 500002) if x != home]
    r.shuffle(locs)
    days = [r.randint(1, 500000) if i % 2 else r.randint(1, 50) for i in range(n)]
    print(n, 10, 1, home)
    print("\n".join(f"{t} {x} {r.randint(1, 4000)}" for t, x in zip(days, locs)))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 500000
n = 500000 if big else r.randint(1, S)
top = 500001 if big or r.random() < 0.2 else r.randint(n + 1, 2 * n + 2)
up = 10 if big and r.random() < 0.6 else r.randint(1, 10)
down = r.choice([up, 1, r.randint(1, up)])
home = r.choice([1, top, r.randint(1, top)])
locs = [x for x in r.sample(range(1, top + 1), n + 1) if x != home][:n]
mode = r.randint(0, 3)  # 0 random days, 1 one day for all, 2 every fair its own day, 3 few days
if mode == 0: days = [r.randint(1, 500000 if big else r.randint(1, 2 * n)) for _ in range(n)]
elif mode == 1: days = [r.randint(1, 500000)] * n
elif mode == 2: days = r.sample(range(1, 500001), n)
else:
    pool = [r.randint(1, 500000) for _ in range(r.randint(1, 5))]
    days = [r.choice(pool) for _ in range(n)]
gain = 4000 if r.random() < 0.3 else None
print(n, up, down, home)
for t, x in zip(days, locs):
    print(t, x, gain or r.choice([1, 4000, r.randint(1, 4000)]))
