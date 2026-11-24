"""CCO 2021 P3 inputs: `gen.py seed size`, or `gen.py --max` (N = Q = 8e5, a long random tree, K up
to 1e15). Each room lists its neighbours in a random clockwise order."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 800000 if MAX else r.randint(2, S + 2)
adj = [[] for _ in range(n + 1)]
for v in range(2, n + 1):
    u = r.randint(max(1, v - 3), v - 1) if MAX else r.randint(1, v - 1)
    adj[u].append(v); adj[v].append(u)
q = 800000 if MAX else r.randint(1, 2 * S + 1)
out = [f"{n} {q}"]
for u in range(1, n + 1):
    r.shuffle(adj[u]); out.append(" ".join(map(str, [len(adj[u])] + adj[u])))
top = 10**15 if MAX or r.randint(0, 1) else 4 * n * n
out += [str(r.randint(1, top)) for _ in range(q)]
print("\n".join(out))
