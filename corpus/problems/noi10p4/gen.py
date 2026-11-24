import sys, random
# seed size: n <= size + 1 planes taking off in a hidden order; each k_i is at least its slot,
# rules only point forward in that order, so a schedule exists; --max: n = 2000, m = 10000
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 1999 if mx else int(sys.argv[2])
n = 2000 if mx else r.randint(1, min(2000, S + 1))
m = 10000 if mx else r.randint(0, min(10000, n * (n - 1) // 2, 3 * n))
order = list(range(1, n + 1)); r.shuffle(order)
slot = {p: i + 1 for i, p in enumerate(order)}
k = [r.randint(slot[p], n) for p in range(1, n + 1)]
rules = []
for _ in range(m if n > 1 else 0):
    i, j = sorted(r.sample(range(n), 2))
    rules.append((order[i], order[j]))
print(n, len(rules)); print(*k)
for a, b in rules: print(a, b)
