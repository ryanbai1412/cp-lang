import random
random.seed(1)
tests = []
def add(n, edges, ops):
    tests.append(f"{n} {len(ops)}\n" + "".join(f"{x} {y}\n" for x, y in edges) + " ".join(map(str, ops)) + "\n")
# chain 1 - 2 - ... - n with a 100000-long branch off vertex 2 (vertices k..n-1); the last op hits the branch end
k, L = 50000, 100000; n = k + L
chain = list(range(1, k)) + [n]
edges = [(chain[i], chain[i + 1]) for i in range(k - 1)] + [(2, k)] + [(v, v + 1) for v in range(k, n - 1)]
add(n, edges, [random.randint(2, n) for _ in range(99999)] + [n - 1])
# path with vertex 1 in the middle and n at one end: every op on the other side
n = 100000; order = list(range(2, n)); random.shuffle(order); order = order[:n // 2] + [1] + order[n // 2:] + [n]
add(n, [(order[i], order[i + 1]) for i in range(n - 1)], [order[random.randrange(n // 2)] for _ in range(100000)])
# random tree, random labels
n = 100000; lab = list(range(1, n + 1)); random.shuffle(lab)
add(n, [(lab[random.randint(0, v - 1)], lab[v]) for v in range(1, n)], [random.randint(2, n) for _ in range(100000)])
# star around 1 except a caterpillar holding n
n = 100000; edges = [(1, v) for v in range(2, n // 2)] + [(n // 2 - 1, n // 2)] + [(v, v + 1) for v in range(n // 2, n)]
add(n, edges, [random.randint(2, n) for _ in range(100000)])
# many tiny tests: resetting O(maxn) arrays per test is 5e9 work
rest = 500000 - sum(int(s.split()[0]) for s in tests)
while len(tests) < 10000 and rest >= 2:
    n = min(rest, 5) if len(tests) < 9999 else rest
    edges = [(random.randint(1, v - 1), v) for v in range(2, n + 1)]
    add(n, edges, [random.randint(2, n) for _ in range(min(n, 9))])
    rest -= n
print(len(tests)); print("".join(tests), end="")
