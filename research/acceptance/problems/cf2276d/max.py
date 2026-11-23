import random
random.seed(1)
N = M = 300000
tests = []
def add(n, es):
    p = list(range(1, n + 1)); random.shuffle(p)
    tests.append((n, [(p[u - 1], p[v - 1]) if random.random() < 0.5 else (p[v - 1], p[u - 1]) for u, v in es]))
# 9996 tiny stars (answer 2): punishes per-test resets of max-size arrays
for _ in range(9996): add(3, [(1, 2), (1, 3)])
add(100000, [(1, v) for v in range(2, 100001)])                      # star: 2
add(90000, [(1, v) for v in range(2, 90001)] + [(2, 3)])             # star + edge: Many
add(40000, [(v, v + 1) for v in range(1, 40000)])                    # path: Many
rn = N - sum(n for n, _ in tests); rm = M - sum(len(e) for _, e in tests)
es = {(1, v) for v in range(2, rn)}                                   # vertex rn isolated: Brain Melt
while len(es) < rm:
    u, v = random.sample(range(1, rn), 2); es.add((min(u, v), max(u, v)))
add(rn, sorted(es))
assert len(tests) <= 10**4 and sum(n for n, _ in tests) <= N and sum(len(e) for _, e in tests) <= M
out = [str(len(tests))]
for n, es in tests:
    out.append(f"{n} {len(es)}"); out += [f"{u} {v}" for u, v in es]
print("\n".join(out))
