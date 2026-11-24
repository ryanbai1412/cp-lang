"""CCO 2023 P6 inputs: `gen.py seed size` (up to `size` lengths with small counts, so parity
flips and singles move), or `gen.py --max` (N = Q = 2e5, counts up to 1e9)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 2 * 10**5 if MAX else r.randint(1, max(1, S))
q = 2 * 10**5 if MAX else r.randint(1, max(1, S))
top = 10**9 if MAX else r.choice([3, 6, 10**9])
c = [r.randint(0, top) for _ in range(n)]
print(n, q)
print(*c)
for _ in range(q):
    l = r.randrange(n)
    if r.random() < 0.5:
        v = c[l] + 1 if c[l] == 0 else c[l] + r.choice([-1, 1])
    else:
        v = r.randint(0, top)
    if v == c[l] or v > top:
        v = c[l] - 1 if c[l] > 0 else c[l] + 1
    print(l + 1, v - c[l])
    c[l] = v
