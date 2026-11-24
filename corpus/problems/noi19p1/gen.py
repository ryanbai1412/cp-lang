import sys, random
if sys.argv[1:] == ["--max"]:
    # n = 1e5, m = 2e5, A = 10, B = C = 1e6, times up to 1000, trains crowded on a few hub
    # stations (mode 1) so every arrival queries a large hull of departures
    sys.argv[1:] = [str(next(s for s in range(100) if random.Random(s).randint(0, 2) == 1)), "100000"]

# seed size: n <= size, m <= 2 * size (capped at 1e5 / 2e5); a route 1 -> n always exists
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10**5
n = 10**5 if big else r.randint(2, max(2, S))
m_cap = 2 * 10**5 if big else r.randint(1, 2 * S)
T = 1000 if big or r.random() < 0.5 else r.randint(2, 30)
mode = r.randint(0, 2)
A = 10 if big and mode else r.randint(0, 10)
B = 10**6 if big and mode else r.choice([0, r.randint(0, 10**6), r.randint(0, 5)])
C = 10**6 if big and mode else r.choice([0, r.randint(0, 10**6), r.randint(0, 5)])
trains = []
# a guaranteed chain 1 -> ... -> n with increasing times
hops = r.randint(1, min(n - 1, T // 2, 50))
mids = r.sample(range(2, n), min(hops - 1, n - 2)) if n > 2 else []
path = [1] + mids + [n]
times = sorted(r.sample(range(0, T + 1), 2 * (len(path) - 1)))
for i in range(len(path) - 1):
    trains.append((path[i], path[i + 1], times[2 * i], times[2 * i + 1]))
hub = [1, n] + [r.randint(1, n) for _ in range(20)]
while len(trains) < max(m_cap, len(trains)):
    if mode == 1 or (not big and r.random() < 0.5):
        x, y = r.choice(hub), r.choice(hub)
    else:
        x, y = r.randint(1, n), r.randint(1, n)
    if x == y: continue
    p = r.randint(0, T - 1); q = r.randint(p + 1, min(T, p + (5 if mode == 2 else T)))
    trains.append((x, y, p, q))
r.shuffle(trains)
print(n, len(trains), A, B, C)
for t in trains: print(*t)
