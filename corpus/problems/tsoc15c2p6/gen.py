import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
# `seed size`: n <= size + 1, q <= size + 1 (size >= 1e6 gives n = 1e6, q = 1e5).
seed, S = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
n, q = (10 ** 6, 10 ** 5) if S >= 10 ** 6 else (r.randint(1, S + 1), r.randint(1, S + 1))
top = 10 ** 9 if S >= 10 ** 6 or seed % 2 else 10
print(n, q)
print(*[r.randint(top // 2 if S >= 10 ** 6 else 0, top) for _ in range(n)])
for _ in range(q):
    a = r.randint(1, n); b = r.randint(a, n)
    print(a, b, r.randint(0, 1000 if S >= 10 ** 6 else top // 3))
