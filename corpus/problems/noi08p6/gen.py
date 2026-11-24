"""gen.py SEED SIZE | --max: NOI 2008 Table Tennis Tournament. "0" (the test number, as in the
sample), n = 2^k, an n x n win matrix to two decimals with P[i][j] + P[j][i] = 1 and a zero
diagonal, then k + 1 rising prizes. The statement gives no bounds; the official files reach
n = 256, so a seed takes k <= min(8, SIZE) and either uniform, near-even or two-level
strengths. --max is n = 256 with uniform probabilities and prizes rising tenfold."""
import random, sys


def matrix(r, n, mode):
    P = [[0] * n for _ in range(n)]
    s = [r.random() for _ in range(n)]
    for i in range(n):
        for j in range(i + 1, n):
            if mode == 0:
                p = r.randint(0, 100)
            elif mode == 1:
                p = r.choice([50, 50, 50, 0, 100, r.randint(45, 55)])
            else:
                p = max(0, min(100, round(50 + 60 * (s[i] - s[j]) + r.randint(-10, 10))))
            P[i][j], P[j][i] = p, 100 - p
    return P


if sys.argv[1] == "--max":
    r, k, mode = random.Random(2008), 8, 0
else:
    r = random.Random(int(sys.argv[1]))
    k, mode = r.randint(0, min(8, int(sys.argv[2]))), r.randrange(3)
n = 1 << k
P = matrix(r, n, mode)
if sys.argv[1] == "--max":
    a = [10 ** i for i in range(1, k + 2)]
else:
    a = sorted(r.sample(range(1, 10 ** 6), k + 1))
print(0)
print(n)
for row in P:
    print(" ".join("%d.%02d" % divmod(p, 100) for p in row))
print("\n".join(map(str, a)))
