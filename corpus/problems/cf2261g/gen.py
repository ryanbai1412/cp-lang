import sys, random
if sys.argv[1] == "--max":  # one case n = 4e5 (sum n): all 25 bits set but one, mixed with all-ones
    r = random.Random(0); n = 400000; F = 2**25 - 1
    print(1); print(n); print(*[F ^ (1 << r.randrange(25)) if r.random() < 0.9 else F for _ in range(n)]); sys.exit()
# usage: gen.py seed size; sum n <= min(4e5, 4 * size), a_i < 2^25
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
budget = min(400000, 4 * S)
F = 2**25 - 1
if S >= 100000:
    ns = [budget] if r.random() < 0.7 else [budget // 4] * 4
else:
    ns = [r.randint(1, max(1, budget // 4)) for _ in range(4)]
print(len(ns))
kind = r.choice(["full", "random", "groups", "mix"]) if S >= 100000 else None
for n in ns:
    if kind == "full": a = [F] * n
    elif kind == "random": a = [r.randint(0, F) for _ in range(n)]
    elif kind == "groups": a = [F ^ (1 << (i % 25)) for i in range(n)]
    elif kind == "mix": a = [F ^ (1 << r.randrange(25)) if r.random() < 0.9 else F for _ in range(n)]
    else:
        hi = r.choice([3, 15, F]); base = r.choice([0, hi])
        a = [(r.randint(0, hi) | (base if r.random() < 0.7 else 0)) for _ in range(n)]
    print(n); print(*a)
