import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); m = random.randint(2 if n == 1 else 1, s); p = random.random() * 0.5
g = [['#' if random.random() < p else '.' for _ in range(m)] for _ in range(n)]
cells = random.sample([(i, j) for i in range(n) for j in range(m)], 2)
g[cells[0][0]][cells[0][1]] = 'A'; g[cells[1][0]][cells[1][1]] = 'B'
print(n, m); print("\n".join("".join(r) for r in g))
