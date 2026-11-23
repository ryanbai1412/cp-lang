import sys, random
# usage: gen.py seed size; prints "n x y", the 2n brackets, then the n weights
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 400000 if S >= 400000 else r.randint(2, max(2, S))
x, y = r.randint(0, 1), r.randint(0, 1)
shape = r.choice(["random", "flat", "deep", "comb"])
s = []
if shape == "flat": s = ["()"] * n
elif shape == "deep": s = ["("] * n + [")"] * n
elif shape == "comb":
    k = r.randint(1, n); s = ["("] * k + ["()"] * (n - k) + [")"] * k
else:
    opened = depth = 0
    while len(s) < 2 * n:
        if opened < n and (depth == 0 or r.random() < 0.5): s.append("("); opened += 1; depth += 1
        else: s.append(")"); depth -= 1
hi = r.choice([3, 10, 10**7]) if S < 400000 else 10**7
lo = hi if r.random() < 0.3 else 1
print(n, x, y); print("".join(s)); print(*[r.randint(lo, hi) for _ in range(n)])
