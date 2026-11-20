import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 3); print(t)
for _ in range(t):
    print(r.randint(1, 5 * S), r.choice([998244353, 10 ** 9 + 7, 10 ** 9 + 9, 999999937]))
