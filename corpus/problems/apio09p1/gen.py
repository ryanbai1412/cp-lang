import sys, random
if sys.argv[1:] == ["--max"]:
    g = random.Random(1)
    n = m = 1500
    k = 300
    print(n, m, k)
    for _ in range(n): print(*(g.randint(0, 500) for _ in range(m)))
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
def fits(n, m, k): return (3 * k <= n and k <= m) or (3 * k <= m and k <= n) or (2 * k <= n and 2 * k <= m)
while True:
    k = r.randint(1, max(1, S // 3))
    n = r.randint(k, max(k, S)); m = r.randint(k, max(k, S))
    if fits(n, m, k): break
top = r.choice([1, 5, 500])
print(n, m, k)
for _ in range(n): print(*(r.randint(0, top) for _ in range(m)))
