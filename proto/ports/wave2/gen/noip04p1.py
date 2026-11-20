import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
top = r.choice([200, 300, 350])
for _ in range(12): print(r.randint(0, top))
