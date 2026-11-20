import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, 3 * S + 1))
