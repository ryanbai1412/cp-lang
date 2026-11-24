import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0)
    print("9" * 10000); print(100)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, 10 ** r.randint(1, 4 * S))); print(r.randint(1, r.choice([3, 100])))
