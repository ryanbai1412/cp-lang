import sys, random
if sys.argv[1] == "--max":
    print(2000); print("\n".join(["1837836000 1 1 1837836000"] * 2000)); sys.exit()
from math import gcd
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 5); print(t); M = r.choice([30, 1000, 40000])
for _ in range(t):
    x = r.randint(1, M); a0 = r.randint(1, M); b0 = r.randint(1, M)
    a1 = gcd(x, a0); b1 = x // gcd(x, b0) * b0
    if r.randint(0, 4) == 0: a1 = r.choice([d for d in range(1, a0 + 1) if a0 % d == 0][:50])
    print(a0, a1, b0, b1)
