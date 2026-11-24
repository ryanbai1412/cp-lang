import sys, random
# seed size: T <= 3 cases of n in [3, size + 3] (at most 1000) piles, k <= min(100, size), ranges either
# tiny (r <= 5), fixed (l = r) or wide up to 1e9; --max: T = 10 cases of n = 1000, k = 100, ranges
# random within [0, 1e9]
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
T = 10 if mx else r.randint(1, 3)
out = [str(T)]
for _ in range(T):
    n = 1000 if mx else r.randint(3, min(1000, S + 3))
    k = 100 if mx else r.randint(0, min(100, S))
    kind = "wide" if mx else r.choice(["tiny", "tiny", "fixed", "wide"])
    cap = 5 if kind == "tiny" else r.choice([10, 10**9])
    out.append(f"{n} {k}")
    for _ in range(n):
        lo, hi = sorted((r.randint(0, cap), r.randint(0, cap)))
        if kind == "fixed":
            hi = lo
        out.append(f"{lo} {hi}")
print("\n".join(out))
