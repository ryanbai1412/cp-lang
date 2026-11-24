import sys, random
if sys.argv[1] == "--max":  # n = m = 500, the tree a path (every vertex but the leaf internal)
    print(500, 500); print(*range(1, 500)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 2 * S + 1); m = r.randint(0, 2 * S + 1); k = r.choice([1, 2, n])
print(n, m); print(*[r.randint(max(1, v - k), v - 1) for v in range(2, n + 1)])
