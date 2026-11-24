"""gen.py SEED SIZE | --max: Mock CCC '22 C1 J1. N, i, j <= 1e4, i != j, and i^2 and j^2
are never equally close to N. Small sizes keep the three numbers small so ties are near;
--max is N = i = 1e4 against j = 1e4 - 1."""
import random, sys
if sys.argv[1] == "--max":
    print(10_000, 10_000, 9_999, sep="\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
hi = min(10_000, 3 + int(sys.argv[2]) ** 2)
while True:
    n, i, j = (r.randint(1, hi) for _ in range(3))
    if i != j and abs(i * i - n) != abs(j * j - n):
        break
print(n, i, j, sep="\n")
