import sys, random
# seed size: n <= 3 rows, m <= size columns, T random (sometimes all 0 or a few 1s); L is T read along
# a random Hamiltonian path from the border when one is found quickly, else random, and sometimes
# one entry flipped; --max: n = 3, m = 50, all T and L 0, the most routes
sys.setrecursionlimit(10000)
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 3 if mx else r.randint(1, 3)
m = 50 if mx else r.randint(1, max(1, min(50, S)))
N = n * m
p = 0 if mx else r.choice([0, 0.1, 0.5, 0.5])
T = [[int(r.random() < p) for _ in range(m)] for _ in range(n)]


def walk():
    border = [(i, j) for i in range(n) for j in range(m) if i in (0, n - 1) or j in (0, m - 1)]
    for _ in range(200):
        path = [r.choice(border)]; seen = set(path)
        while len(path) < N:
            i, j = path[-1]
            nb = [(a, b) for a, b in ((i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1))
                  if 0 <= a < n and 0 <= b < m and (a, b) not in seen]
            if not nb:
                break
            path.append(r.choice(nb)); seen.add(path[-1])
        if len(path) == N:
            return path
    return None


path = None if mx else walk()
L = [T[i][j] for i, j in path] if path else [int(r.random() < p) for _ in range(N)]
if not mx and r.random() < 0.2:
    L[r.randrange(N)] ^= 1
print(n, m)
for row in T:
    print(*row)
print(*L)
