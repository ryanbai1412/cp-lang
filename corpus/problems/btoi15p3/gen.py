import sys, random
# seed size: a tree on 3..=size vertices (capped at 5e5), shuffled labels and edge order;
# shapes: random parent, path, star, caterpillar, broom, deep binary, random with long tails
# --max: n = 5e5, a path of 2.5e5 with a leaf hung on every other vertex past it (deep and
# 1.25e5 leaves), labels and edges shuffled.
if sys.argv[1] == "--max":
    rng, n = random.Random(0), 500000
    par = [i - 1 if i < n // 2 or i % 2 else i - 1 - 1 for i in range(n)]
    lab = list(range(1, n + 1)); rng.shuffle(lab)
    es = [(lab[i], lab[par[i]]) if rng.random() < 0.5 else (lab[par[i]], lab[i]) for i in range(1, n)]
    rng.shuffle(es)
    print(n)
    print("\n".join(f"{u} {v}" for u, v in es))
    sys.exit()
rng = random.Random(int(sys.argv[1])); S = max(3, min(int(sys.argv[2]), 500000))
n = S if rng.random() < 0.3 else rng.randint(3, S)
kind = rng.randrange(7)
par = [0] * n
for i in range(1, n):
    if kind == 0: par[i] = rng.randrange(i)
    elif kind == 1: par[i] = i - 1
    elif kind == 2: par[i] = 0
    elif kind == 3: par[i] = i - 1 if i % 2 or i < 2 else i - 2
    elif kind == 4: par[i] = i - 1 if i < n // 2 else n // 2 - 1
    elif kind == 5: par[i] = (i - 1) // 2
    else: par[i] = rng.randrange(max(0, i - 3), i) if rng.random() < 0.8 else rng.randrange(i)
lab = list(range(1, n + 1)); rng.shuffle(lab)
es = [(lab[i], lab[par[i]]) if rng.random() < 0.5 else (lab[par[i]], lab[i]) for i in range(1, n)]
rng.shuffle(es)
print(n)
print('\n'.join(f"{u} {v}" for u, v in es))
