import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if len(sys.argv) > 3 else r.randint(1, S); d = r.choice([1, 2, 3, r.randint(1, n + 1), r.randint(1, 200000)])
shape = sys.argv[3] if len(sys.argv) > 3 else r.choice(["random", "path", "star", "broom", "deep", "binary"])
def parent(i):
    if shape == "path": return i - 1
    if shape == "star": return 0
    if shape == "broom": return i - 1 if i <= n // 2 else r.randint(0, n // 2)
    if shape == "deep": return r.randint(max(0, i - 3), i - 1)
    if shape == "binary": return (i - 1) // 2
    return r.randint(0, i - 1)
if len(sys.argv) > 4: d = int(sys.argv[4])
print(n, d)
for i in range(1, n): print(parent(i))
