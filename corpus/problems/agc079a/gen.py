import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: T <= 125000, 2 <= N <= 250000, 1 <= P_v < v, sum of N <= 250000
MAX_N, MAX_T = 250_000, 125_000

def parents(n, shape):
    if shape == "chain": return [v - 1 for v in range(2, n + 1)]
    if shape == "star": return [1] * (n - 1)
    if shape == "near": return [r.randint(max(1, v - 3), v - 1) for v in range(2, n + 1)]
    if shape == "zigzag": return [v - 1 if v % 2 else max(1, v - 2) for v in range(2, n + 1)]
    return [r.randint(1, v - 1) for v in range(2, n + 1)]

shapes = ["random", "chain", "star", "near", "zigzag"]
if S < 1000:
    ns = [r.randint(2, 4 * S + 1) for _ in range(5)]
else:
    k = r.randrange(4)
    ns = [MAX_N] if k < 2 else [2] * MAX_T if k == 2 else [MAX_N // 50] * 50
print(len(ns))
for n in ns:
    print(n); print(*parents(n, r.choice(shapes)))
