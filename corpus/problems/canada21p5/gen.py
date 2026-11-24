"""gen.py SEED SIZE | --max: Canada Day 2021 P5. Binary strings s, t of length n <= 128,
0 <= m <= n flips a second, k <= 1e9 seconds. --max is n = 128, m = 64, k = 1e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 128 if MAX else r.randint(1, min(128, 1 + 3 * S))
m = 64 if MAX else r.randint(0, n)
k = 10 ** 9 if MAX else r.choice([0, 1, 2, r.randint(0, 10), r.randint(0, 10 ** 9)])
s = "".join(r.choice("01") for _ in range(n))
t = "".join(r.choice("01") for _ in range(n))
print(n, m, k); print(s); print(t)
