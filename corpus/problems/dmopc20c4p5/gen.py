"""gen.py SEED SIZE | --max: DMOPC '20 Contest 4 P5 (Cyclic Cypher). One line `N K`,
1 <= K <= N <= 2^21, with N and K drawn over 2-adic orders so every case of the construction
(and every impossible one) shows up. --max is N = 2^21, K = 3 * 2^19 (> N/2, v2(K) > 0)."""
import random, sys
if sys.argv[1] == "--max":
    print(2**21, 3 * 2**19)
    sys.exit()
r = random.Random(int(sys.argv[1]))
cap = max(1, min(2**21, int(sys.argv[2])))
for _ in range(1000):
    a = r.randint(0, cap.bit_length())
    n = (r.randint(1, max(1, cap >> a)) | 1) << a if r.random() < 0.8 else r.randint(1, cap)
    if 1 <= n <= cap:
        break
else:
    n = cap
b = r.randint(0, n.bit_length())
k = (r.randint(1, max(1, n >> b)) | 1) << b if r.random() < 0.8 else r.randint(1, n)
if not 1 <= k <= n:
    k = r.randint(1, n)
print(n, k)
