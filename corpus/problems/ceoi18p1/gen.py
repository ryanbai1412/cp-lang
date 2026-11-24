import sys, random

if sys.argv[1] == "--max":
    # n = m = 2000, 50 cores each (a 1e5-wide knapsack), rates and prices up to 1e9
    rng = random.Random(1)
    for _ in range(2):
        print(2000)
        for _ in range(2000):
            print(50, rng.randint(1, 10**9), rng.randint(1, 10**9))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
M = r.choice([4, 10**9])
def offers():
    n = r.randint(1, S + 1); print(n)
    for _ in range(n): print(r.randint(1, r.choice([3, 50])), r.randint(1, M), r.randint(1, M))
offers(); offers()
