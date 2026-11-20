import sys, random
from math import gcd
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
op = r.choice(["first", "second"]); t = 5; print(op); print(t)
for _ in range(t):
    while True:
        n = r.randint(1, 2 * S + 1); w = r.randint(0, n * n)
        if gcd(w, n) == 1 and 2 * w != n * n: break
    cells = set(r.sample(range(n * n), w)); print(n)
    for i in range(n): print(''.join('#' if i * n + j in cells else '.' for j in range(n)))
    if op == "first": print(r.randint(1, n), r.randint(1, n))
