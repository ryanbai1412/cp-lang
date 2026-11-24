import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = S if big else r.randint(1, max(1, min(S, 200000)))
top = 10**7 if big else r.choice([100, 10**4, 10**7])
xs = [r.randint(max(1, top - 1000), top) if r.random() < 0.2 else r.randint(1, top) for _ in range(t)]
print(t)
print(*xs)
