import sys, random
# seed size: an n x m <= size x size grid of digits (sometimes few distinct digits), K <= 500 moves,
# l_min <= l_max, c1, c2 in 0..2, F in 0..1, like the official game1..10; --max: 1000 x 1000
# random digits, K = 500, l in 2..18, c1 = c2 = 2, F = 0
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
if mx:
    n = m = 1000; K = 500; lmin, lmax = 2, 18; c1 = c2 = 2; F = 0; digits = "0123456789"
else:
    n, m = r.randint(1, S), r.randint(1, S)
    K = r.randint(1, 500)
    lmax = r.randint(2, max(2, min(n * m, 3 * S)))
    lmin = r.randint(2, lmax)
    c1, c2, F = r.randint(0, 2), r.randint(0, 2), r.randint(0, 1)
    digits = r.choice(["0123456789", "1379", "12", "1"])
print(r.randint(1, 10))
print(n, m, K, lmin, lmax, c1, c2, F)
for _ in range(n):
    print(*(r.choice(digits) for _ in range(m)))
