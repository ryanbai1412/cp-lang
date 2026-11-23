import random
random.seed(1)
tests = []
def add(n, edges, w, x, y, z): tests.append((n, edges, (w, x, y, z)))
# long path: distances near 3n, endpoints far apart
n = 30000; add(n, [(i, i + 1) for i in range(1, n)], 1, n, n, 1)
# complete graph: the most triangles per edge
k = 500; add(k, [(a, b) for a in range(1, k + 1) for b in range(a + 1, k + 1)], 1, 2, k, k - 1)
# star: one huge degree
n = 20000; add(n, [(1, i) for i in range(2, n + 1)], 2, 3, n, n - 1)
# random connected sparse graph
n, m = 10000, 21251
es = {(random.randint(1, i - 1), i) for i in range(2, n + 1)}
while len(es) < m:
    a, b = sorted(random.sample(range(1, n + 1), 2)); es.add((a, b))
es = list(es); random.shuffle(es)
add(n, es, 1, n, n // 2, 7)
# many tiny tests: per-test resets of max-sized arrays
for i in range(4000): add(2, [(1, 2)], 1 + i % 2, 2, 2 - i % 2, 1)
out = [str(len(tests))]
for n, edges, q in tests:
    out.append(f"{n} {len(edges)}"); out.append(" ".join(map(str, q)))
    out.extend(f"{a} {b}" for a, b in edges)
print("\n".join(out))
