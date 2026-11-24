import sys, random

if sys.argv[1] == "--max":
    # t = 500 tests (no sum bound), each n = 100
    rng = random.Random(1)
    print(500)
    for _ in range(500):
        print(100)
        print("".join(rng.choice("<>") for _ in range(99)))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, min(100, size * 4 + 2))
    print(n)
    print("".join(rng.choice("<>") for _ in range(n - 1)))
