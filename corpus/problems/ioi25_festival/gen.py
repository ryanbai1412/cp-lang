# IOI 2025 Festival, sample-grader stdin: N A, then N lines P[i] T[i]
# (1 <= N <= 2e5; 1 <= A, P[i] <= 1e9; 1 <= T[i] <= 4).
# gen.py SEED SIZE: N <= SIZE; shapes follow the subtasks: plain only, T <= 2, all
# affordable, every multiplier shrinking, and mixed, at small and large token counts.
# gen.py --max: N = 2e5, a free run of multipliers that keep A, then 1.5e5 shrinking
# multipliers (the widest DP after the free run) and plain coupons of every price.
import random
import sys

if sys.argv[1] == "--max":
    r = random.Random(0)
    a = 10**9
    rows = [(a // 2, 2) for _ in range(1000)]
    rows += [(r.randint(a * (t - 1) // t + 1, a), t) for t in (r.randint(2, 4) for _ in range(150_000))]
    rows += [(r.choice([r.randint(1, 10**4), r.randint(1, 10**9)]), 1) for _ in range(200_000 - len(rows))]
    r.shuffle(rows)
    print(len(rows), a)
    print("".join(f"{p} {t}\n" for p, t in rows), end="")
    sys.exit()

r = random.Random(int(sys.argv[1]))
n = r.randint(1, max(1, min(200_000, int(sys.argv[2]))))
a = r.randint(1, r.choice([3, 10, 30, 1000, 10**9]))
shape = r.choice(["mixed", "plain", "two", "shrink", "all"])
types = {"plain": [1], "two": [1, 2]}.get(shape, [1, 2, 3, 4])
print(n, a)
for _ in range(n):
    t = r.choice(types)
    if shape == "shrink" and t > 1:  # (a - p) * t < a: every multiplier loses tokens
        p = r.randint(min(a, a * (t - 1) // t + 1), a)
    elif shape == "all":
        p = r.randint(1, max(1, a // 2))
    else:
        p = r.randint(1, min(10**9, r.choice([3, a, 2 * a, 10**9])))
    print(p, t)
