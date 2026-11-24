"""gen.py SEED SIZE: CCC 2022 J1 Cupcake Party. R regular and S small boxes, R, S >= 0, with at
least 28 cupcakes in total. The statement gives no upper bound on R or S, so there is no --max."""
import random, sys
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
hi = 4 + 10 * S
while True:
    a, b = r.randint(0, hi), r.randint(0, hi)
    if 8 * a + 3 * b >= 28:
        break
print(a)
print(b)
