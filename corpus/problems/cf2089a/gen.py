import sys, random

if sys.argv[1] == "--max":
    # t = 10 tests (no sum bound) at and just below n = 1e5
    print(10)
    for i in range(10):
        print(100_000 - i)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    print(rng.randint(2, size * 8 + 3))
