import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(3, S + 4); print(n)
print(''.join(r.choice('abC1'[:r.randint(1, 4)]) for _ in range(n)))
