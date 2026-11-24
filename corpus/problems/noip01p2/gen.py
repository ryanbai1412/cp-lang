import sys, random
if sys.argv[1] == "--max":
    print(200, 6); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
k = r.randint(1, 6); print(r.randint(k, 200), k)
