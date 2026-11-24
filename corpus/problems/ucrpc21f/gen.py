import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "1000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if big else r.randint(1, min(S, 1000))
t = 1000 if big else r.choice([0, 1, r.randint(0, 1000)])
p = 0.5 if big else r.random()
print(n, t)
for _ in range(n): print("".join("b" if r.random() < p else "-" for _ in range(n)))
