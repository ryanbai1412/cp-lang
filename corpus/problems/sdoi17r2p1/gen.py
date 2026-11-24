"""gen.py seed size: SDOI 2017 Dragons and Dungeons. T <= 10, 2 <= X <= 20,
1 <= Y <= 2e5, at most 2 tests with Y > 800, 10 queries 0 <= A <= B <= (X-1)Y.
Queries cluster within a few standard deviations of the mean, plus whole-range and
single-point ones. size >= 2e5 gives T = 10: two tests with Y near 2e5 (or just over
800) and eight with Y = 800."""
import random, sys
if sys.argv[1:] == ["--max"]:
    # T = 10, X = 20: two tests at Y = 2e5 and eight exact ones at Y = 800
    sys.argv[1:] = ['0', '200000']
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
tests = []
if size >= 200000:
    bigy = [200000, 200000, 801, 199999, 100000][seed % 5]
    for i in range(10):
        x = 20 if seed % 2 == 0 else rng.choice([2, 3, 20])
        tests.append((x, bigy if i < 2 else 800))
    rng.shuffle(tests)
else:
    t = rng.randint(1, 10)
    for i in range(t):
        y = rng.randint(1, size) if i >= 2 or rng.random() < 0.5 else rng.randint(801, 3000)
        tests.append((rng.randint(2, 20), y))
print(len(tests))
for x, y in tests:
    print(x, y)
    top = (x - 1) * y
    mean, sd = top / 2, (y * (x * x - 1) / 12) ** 0.5
    for q in range(10):
        kind = rng.random()
        if q == 0:
            a, b = 0, top
        elif kind < 0.2:
            a = b = min(top, max(0, round(mean + rng.uniform(-2, 2) * sd)))
        else:
            a, b = sorted(min(top, max(0, round(mean + rng.uniform(-4, 4) * sd))) for _ in range(2))
        print(a, b)
