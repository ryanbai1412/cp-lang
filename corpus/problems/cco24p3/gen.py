"""gen.py SEED SIZE | --max: CCO 2024 Summer Driving. 2 <= N <= 300000, 1 <= R, A, B <= N, and the
N-1 roads form a tree. SIZE caps N. --max is N = 300000 on a random tree with A = 4 > B = 3."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 300000 if MAX else r.randint(2, max(2, min(300000, S + 1)))
shape = 1 if MAX else r.randrange(5)
par = [0, 0]
for i in range(2, n + 1):
    if shape == 0: p = i - 1
    elif shape == 1: p = r.randrange(1, i)
    elif shape == 2: p = r.randrange(max(1, i - 3), i)
    elif shape == 3: p = r.randrange(1, min(i, 4))
    else: p = (i // 2) if i > 1 else 1
    par.append(p)
lab = list(range(1, n + 1)); r.shuffle(lab)
def small(): return r.randint(1, min(n, r.choice([1, 2, 3, 5, n])))
R = r.randint(1, n)
A, B = (small(), small())
if MAX: A, B = 4, 3
elif r.random() < 0.8 and n > 1 and A <= B:
    B = r.randint(1, max(1, min(n - 1, B)))
    A = r.randint(B + 1, n) if B < n else A
edges = [(lab[i - 1], lab[par[i] - 1]) for i in range(2, n + 1)]
r.shuffle(edges)
sys.stdout.write(f"{n} {R} {A} {B}\n" + "".join(f"{u} {v}\n" if r.random() < .5 else f"{v} {u}\n" for u, v in edges))
