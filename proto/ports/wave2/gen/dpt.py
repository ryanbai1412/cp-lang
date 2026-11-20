import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, 3 * S + 1)
print(n); print(''.join(r.choice('<>') for _ in range(n - 1)))
