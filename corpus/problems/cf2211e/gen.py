import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
P = [2, 3, 5, 7, 11, 13]
def value():
    k = r.random()
    if k < 0.1: return r.randint(10**18 - 10**6, 10**18)
    if k < 0.2: return r.choice([999999999999999989, 2 * 499999999999999989 % 10**18 or 2])
    x = 1
    for _ in range(r.randint(1, 3)):
        p = r.choice(P[:r.randint(1, 6)])
        if x * p <= 10**18: x *= p
    return max(x, 2)
total = r.randint(2, 6 * S + 2)
ns = []
while total >= 2:
    n = r.randint(2, total)
    if total - n == 1: n = total
    ns.append(n); total -= n
print(len(ns))
for n in ns:
    kids = [[] for _ in range(n + 1)]
    for v in range(2, n + 1):
        lo = 1 if r.random() < 0.5 else max(1, v - 3)
        kids[r.randint(lo, v - 1)].append(v)
    print(n)
    for j in range(n, 0, -1):
        print(value(), len(kids[j]), *kids[j])
