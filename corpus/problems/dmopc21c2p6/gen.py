import sys, random
# statement: a permutation of 1..N and Q queries, N, Q <= 3e5; kinds 1, 2 i, 3 x.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "300000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 300000
n = 300000 if big else r.randint(1, max(1, min(300000, S)))
q = 300000 if big else r.randint(1, max(1, min(300000, S)))
p = list(range(1, n + 1)); r.shuffle(p)
print(n, q)
print(*p)
for _ in range(q):
    t = r.randint(1, 3)
    print(1 if t == 1 else f"{t} {r.randint(1, n)}")
