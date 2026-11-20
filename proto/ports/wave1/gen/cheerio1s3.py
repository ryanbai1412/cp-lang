import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S); print(n)
xs = sorted(r.sample(range(-20, 20), n))
for x in xs: print(x, r.randint(-6, 6))
