import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 1.5e5; starts with 1 and ends with 0, so the longest construction runs
    rng = random.Random(1)
    n = 150_000
    s = ["1"] + [rng.choice("01") for _ in range(2 * n - 1)] + ["0"]
    print(1)
    print(n)
    print("".join(s))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    half = rng.randint(1, size * 3 + 2)
    print(half)
    print("".join(rng.choice("01") for _ in range(2 * half + 1)))
