import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "1000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if big else r.randint(1, max(1, min(S, 1000)))
top = 10**9 if big else r.choice([3, n, 10**9])
print(n)
print(*(r.randint(0, top) for _ in range(n)))
