import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5: P's mostly before B's, so inversions near n^2 / 4 = 1e10
    rng = random.Random(1)
    n = 200_000
    s = ["P" if (i < n // 2) != (rng.random() < 0.05) else "B" for i in range(n)]
    print(1)
    print(n)
    print("".join(s))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, size * 4 + 2)
    print(n)
    print("".join(rng.choice("PB") for _ in range(n)))
