import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
total = r.randint(1, min(1000, 40 * S))
t = r.randint(1, min(100, total))
ns = [1] * t
for _ in range(total - t): ns[r.randrange(t)] += 1
print("first"); print(t)
for n in ns:
    hi = r.choice([2, 1000, 10**6])
    print(n); print(*(r.randint(1, hi) for _ in range(n)))
