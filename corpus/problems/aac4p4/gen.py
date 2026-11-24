import sys, random
# seed size: n, m <= size (capped at 2000), 0 <= L <= R <= n m, 1 <= s, f, k_i <= n
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "2000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 2000
n = 2000 if big else r.randint(1, S)
m = 2000 if big else r.randint(1, S)
s, f = r.randint(1, n), r.randint(1, n)
mode = r.randint(0, 3)
if mode == 0: L = 0
elif mode == 1: L = r.randint(0, n)
elif mode == 2: L = n * m - r.randint(0, m)
else: L = r.randint(0, n * m)
R = r.choice([L, n * m, r.randint(L, n * m)])
k = [r.randint(1, n) for _ in range(m)]
if r.random() < 0.2: k = [x if x != f else (x % n) + 1 for x in k] if n > 1 else k
elif r.random() < 0.5: k[r.randint(0, m - 1)] = f
if r.random() < 0.3: s = f
print(n, m, s, f, L, R)
print(*k)
