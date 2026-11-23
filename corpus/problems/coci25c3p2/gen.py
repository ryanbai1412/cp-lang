import sys, random
# `seed size`: n <= size (size >= 5000 gives n = 5000), 1 <= k <= n.
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 5000 if S >= 5000 else r.randint(1, S)
print(n, r.choice([1, 2, n, max(1, n - 1), r.randint(1, n)]))
