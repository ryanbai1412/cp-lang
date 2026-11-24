"""gen.py SEED SIZE | --max: BSSPC '21 S5 James Solves His Tree Problems. 1 <= Q <= 2000
operations played on a random hidden tree (each D cuts an edge at a leaf, so the tree stays one
piece), the D endpoints relabelled to exactly 1..X; some cases are then perturbed (an operation
dropped, swapped or duplicated) so they may have no tree. Small sizes keep the hidden tree under
ten vertices; a spider keeps the tree alive through long runs of Ls. --max is 1000 Ls and then
1000 Ds cutting a star's leaves, so every hidden vertex hangs a 1000-long chain: the answer has
about a million vertices."""
import random, sys
MAX = sys.argv[1] == "--max"
if MAX:
    print(2000)
    print("L\n" * 1000 + "\n".join(f"D 1 {i}" for i in range(2, 1002)))
    sys.exit()
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
Q = r.randint(1, min(2000, 2 + 3 * S))
T = r.randint(1, min(4000, 9 if S <= 3 else 4 * S))
shape = r.choice(["random", "path", "star", "caterpillar", "spider"])
legs = r.randint(1, 4)
par = [-1] + [r.randrange(i) if shape == "random" else i - 1 if shape == "path"
              else 0 if shape == "star" else (i - 1) // 2 * 2 if shape == "caterpillar"
              else 0 if i <= legs else i - legs for i in range(1, T)]
adj = [set() for _ in range(T)]
for i in range(1, T):
    adj[i].add(par[i])
    adj[par[i]].add(i)
alive = set(range(T))
pl = r.choice([0.0, 0.1, 0.3, 0.6])
ops = []
while len(ops) < Q and alive:
    cut = [(v, w) for v in alive if len(adj[v]) == 1 for w in adj[v]]
    if cut and r.random() >= pl:
        v, w = r.choice(cut)
        adj[v].discard(w)
        adj[w].discard(v)
        for z in (v, w):
            if not adj[z]:
                alive.discard(z)
        ops.append((v, w) if r.random() < 0.5 else (w, v))
    else:
        for v in [v for v in alive if len(adj[v]) <= 1]:
            alive.discard(v)
            for w in adj[v]:
                adj[w].discard(v)
            adj[v].clear()
        ops.append(None)
if ops and r.random() < 0.3:
    i, j = r.randrange(len(ops)), r.randrange(len(ops))
    kind = r.randrange(3)
    if kind == 0 and len(ops) > 1:
        ops.pop(i)
    elif kind == 1:
        ops[i], ops[j] = ops[j], ops[i]
    elif len(ops) < 2000:
        ops.insert(j, ops[i])
label = {}
for op in ops:
    for v in op or ():
        label.setdefault(v, len(label) + 1)
perm = list(range(1, len(label) + 1))
r.shuffle(perm)
print(len(ops))
for op in ops:
    print("L" if op is None else f"D {perm[label[op[0]] - 1]} {perm[label[op[1]] - 1]}")
