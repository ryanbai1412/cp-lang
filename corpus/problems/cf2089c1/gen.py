"""gen.py seed size: CF 2089C1. 1 <= t <= 100, 1 <= n <= 100, 1 <= l <= 5000, k = 0,
sum l <= 5000. size >= 5000 gives max shapes by seed: one test n = 100, l = 5000;
100 tests splitting l; n = 1 with l = 5000."""
import random, sys
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size >= 5000:
    shape = seed % 3
    if shape == 0: tests = [(100, 5000)]
    elif shape == 1: tests = [(100 - i % 7, 50) for i in range(100)]
    else: tests = [(1, 5000)]
else:
    tests = [(rng.randint(1, min(100, size * 2 + 2)), rng.randint(1, size * 3 + 2)) for _ in range(rng.randint(1, 4))]
print(len(tests))
for n, l in tests:
    print(n, l, 0)
