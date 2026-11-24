"""wossoly21s3 generator: an R x C grid (R, C <= 200) of '.', '#', one each of the classes 0-4,
and portal pairs: a capital letter and its small letter, each at most once."""
import random
import string
import sys


def emit(rng, r, c, walls, pairs):
    g = [["#" if rng.random() < walls else "." for _ in range(c)] for _ in range(r)]
    cells = [(i, j) for i in range(r) for j in range(c)]
    picks = rng.sample(cells, min(len(cells), 5 + 2 * pairs))
    for d, (i, j) in zip("01234", picks):
        g[i][j] = d
    letters = rng.sample(string.ascii_lowercase, pairs)
    for k, ch in enumerate(letters):
        if 5 + 2 * k + 1 < len(picks):
            (i, j), (x, y) = picks[5 + 2 * k], picks[5 + 2 * k + 1]
            g[i][j], g[x][y] = ch.upper(), ch
    print(r, c)
    print("\n".join("".join(row) for row in g))


if sys.argv[1] == "--max":
    # A serpentine maze: every leg walks most of the 40000 cells.
    r = c = 200
    g = [["." for _ in range(c)] for _ in range(r)]
    for i in range(1, r, 2):
        for j in range(c):
            g[i][j] = "#"
        g[i][c - 1 if i % 4 == 1 else 0] = "."
    g[0][0], g[r - 2][c - 1], g[0][c - 1], g[r - 2][0], g[r // 2][c // 2] = "0", "1", "2", "3", "4"
    print(r, c)
    print("\n".join("".join(row) for row in g))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
r, c = rng.randint(1, max(1, min(200, size))), rng.randint(1, max(1, min(200, size)))
while r * c < 5:
    r, c = r + 1, c + 1
emit(rng, r, c, rng.choice([0.0, 0.2, 0.4]), rng.randint(0, min(26, max(0, (r * c - 5) // 2))))
