import sys, random
if sys.argv[1] == "--max":
    # N = M = K = 3e5: the existing tubes run all N buildings round one cycle, so the answer is
    # NO only once the cycle is walked, and the bans fall densely among 775 of them.
    # --max --lone L: M = N - L - 1, one (N - L)-building path (deepest DFS) plus L lone buildings
    # banned from each other almost densely, so the complement DFS scans every ban; each lone
    # building can still hang below the path, so the answer is a tree.
    rng = random.Random(0); n = 300000
    lone = int(sys.argv[3]) if sys.argv[2:3] == ["--lone"] else 0
    lab = list(range(n)); rng.shuffle(lab)
    path, solo = lab[:n - lone], lab[n - lone:] if lone else lab[:775]
    must = [(path[i], path[i + 1]) for i in range(len(path) - 1)] + ([] if lone else [(path[-1], path[0])])
    tube = set(must)
    ban = [(a, b) for a in solo for b in solo if a != b and (a, b) not in tube]
    rng.shuffle(ban); ban = ban[:300000]
    rng.shuffle(must)
    out = [f"{n} {len(must)} {len(ban)}"] + [f"{a} {b}" for a, b in must + ban]
    print("\n".join(out)); sys.exit()
# seed size: N in 2..=size (capped at 3e5); a random in-tree on shuffled labels, a random share
# of its arcs as the existing tubes (sometimes broken by a cycle or a second outgoing tube), then
# banned pairs: sparse random, dense on small N, or a few roots banned from all but a few trees.
rng = random.Random(int(sys.argv[1])); S = max(2, min(int(sys.argv[2]) if len(sys.argv) > 2 else 12, 300000))
n = rng.randint(2, S)
lab = list(range(n)); rng.shuffle(lab)
par = [None] + [rng.randrange(i) if rng.random() < 0.7 else i - 1 for i in range(1, n)]
keep = rng.random()
must = [(lab[i], lab[par[i]]) for i in range(1, n) if rng.random() < keep]
used = set(must)
kind = rng.randrange(6)
if kind == 1 and n > 2:  # the tree's root gets a tube into its own tree: a cycle, if that path is kept
    a, b = lab[0], lab[rng.randrange(1, n)]
    if (a, b) not in used: must.append((a, b)); used.add((a, b))
elif kind == 2 and must and n > 2:  # a second outgoing tube
    a, b = rng.choice(must)
    c = rng.choice([v for v in range(n) if v not in (a, b)])
    if (a, c) not in used: must.append((a, c)); used.add((a, c))
up = {a: b for a, b in must}
def root(v):
    seen = set()
    while v in up and v not in seen: seen.add(v); v = up[v]
    return v
ban = []
def add(a, b):
    if a != b and (a, b) not in used and len(ban) < 300000: used.add((a, b)); ban.append((a, b))
if kind == 3 and n <= 60:
    q = rng.uniform(0.3, 0.97)
    for a in range(n):
        for b in range(n):
            if rng.random() < q: add(a, b)
elif kind == 4:
    tops = [v for v in range(n) if v not in up]
    tree = {}
    for v in range(n): tree.setdefault(root(v), []).append(v)
    for c in rng.sample(tops, min(len(tops), rng.randint(1, 4))):
        ok = set(rng.sample(tops, min(len(tops), rng.randint(0, 2))))
        for t in tops:
            if t not in ok:
                for x in tree[t]: add(c, x)
else:
    for _ in range(rng.randint(0, min(3 * n, n * (n - 1)))): add(rng.randrange(n), rng.randrange(n))
rng.shuffle(must); rng.shuffle(ban)
print(n, len(must), len(ban))
for a, b in must + ban: print(a, b)
