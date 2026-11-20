import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
k = r.randint(1, 6); print(r.randint(k, 200), k)
