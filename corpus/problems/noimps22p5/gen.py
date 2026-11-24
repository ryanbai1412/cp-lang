import sys, random

if sys.argv[1] == "--max":
    # n = 4e5, x = 1, y = 0 (the two-lightest branch), all n brackets side by side at depth 1, so
    # every level below keeps all but one of them: n levels of shrinking bags
    rng = random.Random(1)
    n = 400_000
    print(n, 1, 0); print("()" * n); print(*[rng.randint(1, 10**7) for _ in range(n)])
    sys.exit()
# usage: gen.py seed size; prints "n x y", the 2n brackets, then the n weights
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 400000 if S >= 400000 else r.randint(2, max(2, S))
x, y = r.choice([(0, 1), (1, 0), (1, 1)])  # the judge never has x = y = 0
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
