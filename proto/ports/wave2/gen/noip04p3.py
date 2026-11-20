import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 3); print(n)
print(*[r.randint(130, r.choice([140, 230])) for _ in range(n)])
