import sys, random
from math import isqrt
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
def need(p, q):
    m = isqrt(4 * (p * p + q * q))
    return m if m * m == 4 * (p * p + q * q) else m + 1
cases = []
if size > 100:
    budget = 100000
    while True:
        p, q = (10000, 10000) if not cases else (rng.randint(1, 10000), rng.randint(1, 10000))
        if budget < need(p, q) or len(cases) == 1000: break
        cases.append((p, q)); budget -= need(p, q)
    while len(cases) < 1000 and budget >= need(1, 1):
        p, q = rng.randint(1, 20), rng.randint(1, 20)
        if need(p, q) <= budget: cases.append((p, q)); budget -= need(p, q)
else:
    hi = rng.choice([3, 10 * size, 100 * size])
    cases = [(rng.randint(1, hi), rng.randint(1, hi)) for _ in range(rng.randint(1, 10))]
print(len(cases))
for p, q in cases: print(p, q, need(p, q))
