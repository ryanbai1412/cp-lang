"""gen.py SEED SIZE | --max: DMOPC '20 Contest 4 P6 (Petty Children). A simple graph on
1 <= N <= 2000 vertices with any M <= N(N-1)/2, edges listed in random order; densities range
from empty to near-complete. --max is N = 2000 with all but 1000 random pairs as edges."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 2000 if MAX else r.randint(1, max(1, min(2000, int(sys.argv[2]))))
pairs = [(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)]
if MAX:
    drop = set(r.sample(range(len(pairs)), 1000))
    edges = [p for i, p in enumerate(pairs) if i not in drop]
else:
    p = r.choice([0.0, 0.1, 0.3, 0.5, 0.9, 1.0])
    edges = [e for e in pairs if r.random() < p]
r.shuffle(edges)
edges = [(b, a) if r.random() < 0.5 else (a, b) for a, b in edges]
sys.stdout.write(f"{n} {len(edges)}\n" + "".join(f"{a} {b}\n" for a, b in edges))
