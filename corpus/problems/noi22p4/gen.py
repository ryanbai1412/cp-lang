"""noi22p4 (Challenge NPC II) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"C T k", then per test G and H, each as n then n parents (-1 for the root), with
|H| <= |G| <= |H| + k, |G| <= 1e5 and sum |G| <= 5e5. H is G less up to k leaves, sometimes
with one leaf moved (often No). --max: T = 5 trees of 1e5 with k = 5, deep and bushy shapes so
many subtrees share a shape and the near-misses go deep.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 5 * 10**5 if MAX else max(2, int(sys.argv[2]))
k = 5 if MAX else rng.randint(0, 5)
tests = 5 if MAX else rng.randint(1, 5)
per = size // tests if MAX else max(1, size // tests)


def tree(n):
    par = [-1] * n
    kind = rng.randrange(3)
    for i in range(1, n):
        if kind == 0:
            par[i] = rng.randrange(max(0, i - 3), i)
        elif kind == 1:
            par[i] = (i - 1) // 2 if rng.random() < 0.9 else rng.randrange(i)
        else:
            par[i] = rng.randrange(i)
    return par


def shrink(par, drop):
    """par with `drop` leaves removed, as a parent list over the kept nodes."""
    n = len(par)
    kids = [0] * n
    for p in par[1:]:
        kids[p] += 1
    gone = set()
    leaves = [v for v in range(1, n) if kids[v] == 0]
    while len(gone) < drop and leaves:
        v = leaves.pop(rng.randrange(len(leaves)))
        gone.add(v)
        kids[par[v]] -= 1
        if par[v] and kids[par[v]] == 0:
            leaves.append(par[v])
    keep = [v for v in range(n) if v not in gone]
    at = {v: i for i, v in enumerate(keep)}
    return [-1 if par[v] < 0 else at[par[v]] for v in keep]


def show(par, out):
    n = len(par)
    lab = list(range(n))
    rng.shuffle(lab)
    line = [0] * n
    for v in range(n):
        line[lab[v]] = -1 if par[v] < 0 else lab[par[v]] + 1
    out.append(str(n))
    out.append(" ".join(map(str, line)))


out = [f"{rng.randint(1, 20)} {tests} {k}"]
for _ in range(tests):
    n = per if MAX else rng.randint(1, per)
    g = tree(n)
    h = shrink(g, rng.randint(0, min(k, n - 1)) if not MAX else min(k, n - 1))
    if len(h) > 2 and rng.random() < 0.5:
        v = rng.randrange(1, len(h))
        if v not in h:
            h[v] = rng.randrange(v)
    show(g, out)
    show(h, out)
print("\n".join(out))
