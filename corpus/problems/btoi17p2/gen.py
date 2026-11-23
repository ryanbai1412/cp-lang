# BOI 2017 Toll: `gen.py SEED S`. Roads only from layer a / K to the next, one per pair,
# tolls 1..=10000, orders 0 <= a < b < N (statement constraints).
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
k = r.randint(1, 5)
n = r.randint(2, max(2, S * k))
pairs = [(a, b) for a in range(n) for b in range((a // k + 1) * k, min(n, (a // k + 2) * k))]
m = r.randint(0, len(pairs)) if r.random() < 0.3 else int(len(pairs) * r.choice([0.3, 0.7, 1.0]))
roads = r.sample(pairs, m)
high = r.choice([3, 10000])
o = r.randint(1, 2 * S)
froms_zero = r.random() < 0.2
print(k, n, m, o)
for a, b in roads: print(a, b, r.randint(1, high))
for _ in range(o):
    a = 0 if froms_zero else r.randint(0, n - 2)
    print(a, r.randint(a + 1, n - 1))
