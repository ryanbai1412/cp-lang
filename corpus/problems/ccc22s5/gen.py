"""gen.py SEED SIZE | --max: CCC 2022 S5. A tree of 2 <= n <= 2e5 students, a Y/N string with at
least one of each, costs 1..=1000. Half the trees are deep (parent within 3 of the child). --max is
n = 2e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(2, 3 + 3 * S)
deep = r.random() < .5
par = [r.randint(max(1, i - 3) if deep else 1, i - 1) for i in range(2, n + 1)]
ids = list(range(1, n + 1)); r.shuffle(ids)
edges = [(ids[p - 1], ids[i - 1]) for i, p in zip(range(2, n + 1), par)]
r.shuffle(edges)
y = r.choice([.2, .5, .8])
s = ["Y" if r.random() < y else "N" for _ in range(n)]
s[r.randrange(n)] = "Y"
while "N" not in s: s[r.randrange(n)] = "N"
hi = 1000 if MAX or r.random() < .5 else 4
out = [str(n)] + [f"{u} {v}" for u, v in edges] + ["".join(s)]
out += [" ".join(str(r.randint(1, hi)) for _ in range(n))]
sys.stdout.write("\n".join(out) + "\n")
