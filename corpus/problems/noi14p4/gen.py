import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t): print(''.join(r.choice("ab" if r.randint(0, 2) else "a") for _ in range(r.randint(1, 5 * S))))
