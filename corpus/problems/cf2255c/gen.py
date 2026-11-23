import sys, random
from math import gcd
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print("first"); print(t)
for _ in range(t):
    while True:
        n = r.randint(2, 2 * S + 1); w = r.randint(0, n * n)
        if gcd(w, n) == 1 and 2 * w != n * n: break
    cells = set(r.sample(range(n * n), w)); print(n)
    for i in range(n): print(''.join('#' if i * n + j in cells else '.' for j in range(n)))
    print(r.randint(1, n), r.randint(1, n))
