import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "500000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if big else r.randint(1, max(1, min(S, 500000)))
deep = 0.9 if big else r.random()
p = r.choice([0.5, 0.6, 0.4])
print(n)
print("".join("(" if r.random() < p else ")" for _ in range(n)))
if n > 1: print(*(i - 1 if r.random() < deep else r.randint(1, i - 1) for i in range(2, n + 1)))
