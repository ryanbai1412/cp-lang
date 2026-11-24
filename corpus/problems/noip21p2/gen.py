import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "30"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 30 if big else r.randint(1, max(1, min(S, 30)))
m = 100 if big else r.randint(0, max(0, min(S, 100)))
k = n if big else r.randint(1, n)
print(n, m, k)
print(*(r.randint(0, 998244352) if big or r.random() < 0.5 else r.randint(0, 3) for _ in range(m + 1)))
