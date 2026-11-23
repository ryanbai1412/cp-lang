import sys, random
# usage: gen.py seed size; a connected simple graph, solvable unless some wires get flipped
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 200000: n, m = 200000, 500000
else:
    n = r.randint(1, max(1, min(S, 200000)))
    m = r.randint(max(1, n - 1), max(1, min(500000, n * (n - 1) // 2, 3 * n)))
if n == 1: n = 2; m = 1
edges = set()
for v in range(1, n):
    p = r.randint(max(0, v - r.choice([1, 5, v])), v - 1); edges.add((p, v))
while len(edges) < m:
    a, b = r.sample(range(n), 2); edges.add((min(a, b), max(a, b)))
order = list(range(n)); r.shuffle(order); when = [0] * n
for t, v in enumerate(order): when[v] = t
sign = [r.randint(0, 1) for _ in range(n)]
flip = r.choice([0, 0, 1, 3])
out = []
for a, b in edges:
    late = a if when[a] > when[b] else b
    other = b if late == a else a
    to = late if sign[late] else other  # electrons end closer to `to`
    out.append((to, other if to == late else late))
for _ in range(flip):
    i = r.randrange(len(out)); out[i] = out[i][::-1]
r.shuffle(out)
print(n, len(out))
print("\n".join(f"{a + 1} {b + 1}" for a, b in out))
