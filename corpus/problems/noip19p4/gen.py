import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "2000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 100 if big else r.randint(1, max(1, min(S, 8)))
m = 2000 if big else r.randint(1, max(1, min(S, 8)))
top = 998244352 if big or r.random() < 0.3 else r.choice([1, 3])
print(n, m)
for _ in range(n): print(*(r.randint(0, top) if r.random() < 0.8 else 0 for _ in range(m)))
