import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S); m = r.choice([5, 30, 10**4])
print(n)
for _ in range(n): print(r.randint(1, m), r.randint(1, m), r.randint(1, 10**9))
