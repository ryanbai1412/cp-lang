import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 4 * S); print(n)
for _ in range(n): print(r.choice([r.randint(0, 5), 1500000000, r.randint(0, 1500000000)]))
