import sys, random
from math import gcd
# seed size: n <= size columns of height <= 2 size (sometimes all equal or staircase), the sum fixed so
# gcd(4, n) divides it and a solution exists; --max: n = 1500 random heights up to 100
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 1500 if mx else r.randint(1, min(1500, S))
H = 100 if mx else max(1, 2 * S)
shape = "random" if mx else r.choice(["random", "random", "flat", "stairs"])
if shape == "flat":
    h = [r.randint(0, H)] * n
elif shape == "stairs":
    h = [i * H // max(1, n - 1) for i in range(n)]
else:
    h = [r.randint(0, H) for _ in range(n)]
while sum(h) % gcd(4, n) or not any(h):
    h[r.randrange(n)] += 1
print(n); print(*h)
