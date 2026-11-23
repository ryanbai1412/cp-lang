import sys, random
from math import gcd
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: 1 <= N, M <= 150000, 1 <= X <= Y <= N, 1 <= Z <= 16
MAX = 150_000
n = r.randint(1, S + 2) if S < 1000 else MAX
m = r.randint(1, S + 2) if S < 1000 else MAX
a = [r.choice([1, 2, 4, 8, 16, 3, 6, 12, 5, 10, 15, 7, 14, 9, 11, 13]) for _ in range(n)]
sp = [a]
while (1 << len(sp)) <= n:
    p = sp[-1]; h = 1 << (len(sp) - 1)
    sp.append([gcd(p[i], p[i + h]) for i in range(len(p) - h)])
def g(l, rr):
    k = (rr - l + 1).bit_length() - 1
    return gcd(sp[k][l], sp[k][rr - (1 << k) + 1])
bad = r.random() < 0.5
print(n, m)
for _ in range(m):
    l = r.randint(1, n); rr = r.randint(l, min(n, l + r.choice([0, 1, 3, 10, n])))
    z = g(l - 1, rr - 1)
    if bad and r.random() < 1e-4 * (1 if S >= 1000 else 1000): z = r.randint(1, 16)
    print(l, rr, z)
