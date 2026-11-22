import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); print(n)
for _ in range(2):
    print(''.join(r.choice('WL') for _ in range(n)))
    print(*[r.randint(1, r.choice([4, 10 ** 6])) for _ in range(n)])
