import sys, random
# statement: n <= 400 switches, link lengths below 20000; p cores at most.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "400"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 400
n = 400 if big else r.randint(1, max(1, min(400, S)))
p = r.randint(1, n + 2)
if big: p = n
print(n, p)
print(*[r.randint(0, r.choice([10, 1000, 100000])) for _ in range(n)])
for v in range(2, n + 1):
    u = v - 1 if big and r.random() < 0.5 else r.randint(max(1, v - r.choice([1, 3, v])), v - 1)
    print(u, v, r.randint(0, r.choice([5, 19999])))
