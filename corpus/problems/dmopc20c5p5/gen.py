"""gen.py SEED SIZE | --max: DMOPC '20 Contest 5 P5 (Bob and the screen colouring).
2 <= N, M and 4 <= N * M <= 200000; the area is capped at SIZE (at least 4), with square,
thin and random shapes. --max is the square 447 x 447 screen, the most row pairs."""
import random, sys
if sys.argv[1] == "--max":
    print(447, 447)
    sys.exit()
r = random.Random(int(sys.argv[1]))
area = max(4, min(200000, int(sys.argv[2])))
kind = r.choice(["square", "thin", "random"])
if kind == "square":
    n = max(2, int(area ** 0.5))
    m = max(2, area // n)
elif kind == "thin":
    n, m = 2, area // 2
else:
    n = r.randint(2, area // 2)
    m = r.randint(2, area // n)
if r.random() < 0.5:
    n, m = m, n
print(n, m)
