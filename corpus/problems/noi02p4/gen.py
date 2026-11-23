import sys, random
from math import gcd

# The statement promises an answer <= 1e6: draw cavemen until m = 1e6 itself is peaceful.
LIMIT = 10**6


def meets(a, b, m):
    (c1, p1, l1), (c2, p2, l2) = a, b
    da, db = (p1 - p2) % m, (c2 - c1) % m
    g = gcd(da, m)
    if db % g:
        return False
    step = m // g
    x = 0 if step == 1 else db // g * pow(da // g, -1, step) % step
    return x <= min(l1, l2)


def peaceful(men, m):
    return not any(meets(men[i], men[j], m) for i in range(len(men)) for j in range(i))


seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
while True:
    if size <= 20:
        n = rng.randint(1, min(15, size + 1))
        hi = rng.choice([10, 100])
        lmax = rng.choice([0, 3, 20, 1000])
    else:
        n = 15
        hi = 100
        lmax = rng.choice([10**4, 10**5, 10**6])
    men = [(rng.randint(1, hi), rng.randint(1, hi), rng.randint(0, lmax)) for _ in range(n)]
    if peaceful(men, LIMIT):
        break
print(n)
for c, p, l in men:
    print(c, p, l)
