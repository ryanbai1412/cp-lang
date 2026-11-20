import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
M = r.choice([4, 10**9])
def offers():
    n = r.randint(1, S + 1); print(n)
    for _ in range(n): print(r.randint(1, r.choice([3, 50])), r.randint(1, M), r.randint(1, M))
offers(); offers()
