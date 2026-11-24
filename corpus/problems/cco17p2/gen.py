"""gen.py SEED SIZE | --max: CCO 2017 P2. An N x M empire, N, M <= 1e8, that k x 2k districts can
build (small cases pick N, M by growing a random rectangle). --max is a 1e8 x 5e7 empire."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
if MAX:
    print(10 ** 8, 5 * 10 ** 7)
else:
    a, b = r.choice([(1, 2), (2, 1)])
    for _ in range(r.randint(0, 2 + S)):
        if r.random() < .5 and a % 2 == 0 and a // 2 + b <= 300: b += a // 2
        elif 2 * a + b <= 300: b += 2 * a
        a, b = b, a
    print(a, b)
