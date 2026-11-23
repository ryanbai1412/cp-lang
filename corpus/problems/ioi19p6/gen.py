# usage: gen.py SEED [N]: one case in the sample grader's format (`n m`, n lines `x h`,
# m lines `l r y`, `s g`), n and m <= N. Skywalks at one height never overlap except at
# an end building, and y <= min(h[l], h[r]), as the statement requires.
import random, sys

r = random.Random(int(sys.argv[1]))
big = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([5, 10, 30, 200])
n = r.randint(2, big)
m = r.randint(1, big)
xs = sorted(r.sample(range(r.choice([n, 3 * n, 10**9 + 1])), n))
hmax = r.choice([3, 10, 1000, 10**9])
h = [r.randint(1, hmax) for _ in range(n)]
shape = r.choice(["any", "short", "long"])
used = {}  # y -> list of (l, r)
walks = []
for _ in range(20 * m):
    if len(walks) == m:
        break
    a = r.randrange(n)
    if shape == "short":
        b = min(n - 1, a + r.randint(1, 3))
    elif shape == "long":
        b = r.randint(a, n - 1) if r.random() < 0.3 else n - 1
    else:
        b = r.randrange(n)
    a, b = min(a, b), max(a, b)
    if a == b:
        continue
    y = r.randint(1, min(h[a], h[b], r.choice([3, 10, hmax])))
    if any(a < q and p < b for p, q in used.get(y, [])):
        continue
    used.setdefault(y, []).append((a, b))
    walks.append((a, b, y))
if not walks:
    walks.append((0, 1, r.randint(1, min(h[0], h[1]))))
if r.random() < 0.4:
    s, g = 0, n - 1
else:
    s, g = r.sample(range(n), 2)
print(n, len(walks))
for x, hh in zip(xs, h):
    print(x, hh)
for a, b, y in walks:
    print(a, b, y)
print(s, g)
