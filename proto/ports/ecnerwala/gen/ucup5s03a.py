import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, 5 * S * S), r.randint(1, 5 * S * S))
