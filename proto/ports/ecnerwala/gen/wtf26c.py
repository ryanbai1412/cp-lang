import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, min(S + 1, 6)); print(n)
a = sorted(r.randint(0, 10**8) for _ in range(n))
print(*a)
