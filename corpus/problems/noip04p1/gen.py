import sys, random
if sys.argv[1] == "--max":
    print("\n".join(["0"] * 12)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
top = r.choice([200, 300, 350])
for _ in range(12): print(r.randint(0, top))
