import sys, random
# seed size: n <= size + 1 (at most 300) pillars, ranges [A, B] under a random cap (7, 100, 1e4 or
# 1e9), sometimes A = B or A = 1, B = 1e9; --max: n = 300, every range a random wide part of
# [1, 1e9] so the interval breakpoints are all distinct
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 300 if mx else r.randint(1, min(300, S + 1))
cap = 10**9 if mx else r.choice([7, 100, 10**4, 10**9])
kind = "wide" if mx else r.choice(["fixed", "full", "any", "any"])
print(n)
for _ in range(n):
    if kind == "fixed":
        a = b = r.randint(1, cap)
    elif kind == "full":
        a, b = 1, cap
    else:
        a, b = sorted(r.randint(1, cap) for _ in range(2))
        if kind == "wide":
            a, b = r.randint(1, cap // 3), r.randint(2 * cap // 3, cap)
    print(a, b)
