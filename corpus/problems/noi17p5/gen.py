import sys, random
# seed size: n <= size + 1 kinds, m <= 10, k <= size + 1 distinct days p <= 5 size, a, s, c, x of a
# random magnitude (some s or x zero); --max: n = 1e5, m = 10, k = 1e5 (every day 1..1e5, shuffled),
# a, s up to 1e9, c up to 1e9 with small x so most kinds last all the span
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 100000 if mx else r.randint(1, min(100000, S + 1))
m = 10 if mx else r.randint(1, 10)
P = 100000 if mx else r.randint(1, min(100000, 5 * S))
k = P if mx else r.randint(1, min(P, S + 1))
cap = 10**9 if mx else r.choice([3, 10, 10**9])
ns, nx = (False, False) if mx else (r.random() < 0.2, r.random() < 0.2)
print(n, m, k)
for _ in range(n):
    c = r.randint(1, cap)
    x = 0 if nx or r.random() < 0.1 else r.randint(0, max(1, c // r.choice([1, 2, P])))
    print(r.randint(1, cap), 0 if ns else r.randint(0, cap), c, x)
ps = r.sample(range(1, P + 1), k)
print(*ps, sep="\n")
