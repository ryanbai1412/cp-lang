import sys, random
# statement: 1 <= n <= 16, a tree on nodes 0..n given by n edges.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "16"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 16 if S >= 16 else r.randint(1, max(1, min(S, 16)))
perm = list(range(n + 1))
r.shuffle(perm)
print(n)
for i in range(1, n + 1):
    a, b = perm[i], perm[r.randint(0, i - 1)]
    print(*((a, b) if r.random() < 0.5 else (b, a)))
