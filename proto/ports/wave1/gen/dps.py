import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, 10 ** r.randint(1, 4 * S))); print(r.randint(1, r.choice([3, 100])))
