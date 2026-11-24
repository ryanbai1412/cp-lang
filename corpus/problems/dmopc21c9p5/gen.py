"""gen.py SEED SIZE | --max: DMOPC '21 Contest 9 P5 (Chika Circle). T games, each the
neighbour minima of a hidden permutation of 1..N (N >= 3), sum of N at most min(SIZE, 1e6)
(at least one game of N = 3). Hidden permutations are uniform, or put small cards on every
other seat, or are sorted runs, so both forced and free cards appear. --max is one game of
N = 1e6 with small cards on every other seat."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
budget = 10**6 if MAX else max(3, min(10**6, int(sys.argv[2])))


def hidden(n, mode):
    if mode == "uniform":
        p = list(range(1, n + 1))
        r.shuffle(p)
        return p
    if mode == "alternate":
        small = list(range(1, n // 2 + 1))
        big = list(range(n // 2 + 1, n + 1))
        r.shuffle(small), r.shuffle(big)
        p = []
        for i in range(n):
            p.append(small.pop() if i % 2 == 0 and small else big.pop() if big else small.pop())
        return p
    p = list(range(1, n + 1))
    cut = r.randint(0, n)
    p = p[cut:] + p[:cut]
    for _ in range(r.randint(0, 3)):
        i, j = r.randrange(n), r.randrange(n)
        p[i], p[j] = p[j], p[i]
    return p


games = []
left = budget
while left >= 3:
    n = budget if MAX else r.randint(3, max(3, min(left, r.choice([4, 6, 10, 50, left]))))
    mode = "alternate" if MAX else r.choice(["uniform", "alternate", "runs"])
    p = hidden(n, mode)
    games.append([min(p[i - 1], p[(i + 1) % n]) for i in range(n)])
    left -= n
    if MAX or r.random() < 0.3:
        break
sys.stdout.write(f"{len(games)}\n" + "".join(f"{len(a)}\n{' '.join(map(str, a))}\n" for a in games))
