import sys, random

seed, S = (7, 150) if "--max" in sys.argv[1:] else (int(sys.argv[1]), int(sys.argv[2]))
r = random.Random(seed)
full = S >= 8 and seed % 64 == 7
tests = []
left = 50
while left > 0 and len(tests) < (1 if full else 8):
    n = 50 if full else r.randint(1, min(left, 1 + S * 2))
    big = r.random() < 0.6
    m = 10**15 if full else r.randint(1, 10**15 if big else 1 + S * S)
    tests.append((n, m))
    left -= n
print(len(tests))
for n, m in tests:
    print(n, m)
