import random
import sys

if sys.argv[1] == "--max":
    # one test at sum n = 5000 with 70% of the entries blank
    rng = random.Random(1)
    n = 5000
    a = list(range(1, n + 1))
    rng.shuffle(a)
    print(1)
    print(n)
    print(*(0 if rng.random() < 0.7 else x for x in a))
    sys.exit()

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size == 8:
    ns = [5000] if seed % 2 else [rng.randint(1, 50) for _ in range(100)]
else:
    ns = [rng.randint(1, 2 * size) for _ in range(rng.randint(1, 8))]
print(len(ns))
for n in ns:
    a = list(range(1, n + 1))
    rng.shuffle(a)
    hole = rng.random()
    print(n)
    print(*(0 if rng.random() < hole else x for x in a))
