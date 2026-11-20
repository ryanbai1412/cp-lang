import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(1, 4 * S * S); k = r.choice([1, 2, 3, 26])
    print(n); print(''.join(chr(97 + r.randrange(k)) for _ in range(n)))
