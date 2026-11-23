import sys, random
# seed size: T queries; size >= 1e6 gives the worst case (T = 1000, n and m near 1e6)
rng = random.Random(int(sys.argv[1])); S = min(int(sys.argv[2]), 10**6)
big = S >= 10**6
t = 1000 if big else rng.randint(1, 20)
print(t)
for _ in range(t):
    if big: print(10**6 - rng.randint(0, 3), 10**6 - rng.randint(0, 1000))
    else: print(rng.randint(1, S), rng.randint(1, S))
