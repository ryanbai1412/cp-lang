import sys, random
# port: 1 <= n <= 1500, an n x n grid holding 1..=n^2 once each.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1500"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 1500 if S >= 1500 else r.randint(1, max(1, min(S, 1500)))
v = list(range(1, n * n + 1))
if r.random() < 0.8 or S >= 1500:
    r.shuffle(v)
print(n)
print("\n".join(" ".join(map(str, v[i * n:(i + 1) * n])) for i in range(n)))
