import sys, random
if sys.argv[1] == "--max":  # n = k = 250000
    print(250000, 250000); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, 5 * S * S), r.randint(1, 5 * S * S))
