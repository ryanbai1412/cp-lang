import sys, random
if sys.argv[1] == "--max":  # N = 80, A spread up to 998244352
    r = random.Random(0); print(80); print(*sorted(r.sample(range(998244353), 80))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, min(S + 1, 6)); print(n)
a = sorted(r.randint(0, 10**8) for _ in range(n))
print(*a)
