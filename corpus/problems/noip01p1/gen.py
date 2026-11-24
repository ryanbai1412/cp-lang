import sys, random
# seed size: a * (x - r1)(x - r2)(x - r3) with roots of two decimals in [-100, 100], pairwise at
# least 1 apart, and a nonzero integer; --max: the widest roots -100, 99.5, 100 at a = 1
mx = sys.argv[1] == "--max"
if mx:
    a, rs = 1, [-10000, 9950, 10000]
else:
    r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
    w = min(20000, 300 * S)
    while True:
        rs = sorted(r.randint(-w // 2, w // 2) for _ in range(3))
        if rs[1] - rs[0] >= 100 and rs[2] - rs[1] >= 100 and rs[0] >= -10000 and rs[2] <= 10000:
            break
    a = r.choice([-1, 1]) * r.randint(1, 5)
x, y, z = rs  # in hundredths
coef = [a, -a * (x + y + z), a * (x * y + y * z + z * x), -a * x * y * z]
print(*(f"{c / 100 ** i:.{2 * i}f}" if i else str(c) for i, c in enumerate(coef)))
