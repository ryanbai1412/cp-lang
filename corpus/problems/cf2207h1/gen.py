import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
total = r.randint(2, min(70, 4 + 8 * S))
ns = []
while total >= 2:
    n = r.randint(2, total) if r.random() < 0.5 else min(total, r.randint(2, 6))
    if total - n == 1: n = total
    ns.append(n); total -= n
def expr(lo, hi):
    if lo == hi: return [str(lo)]
    m = r.randint(lo, hi - 1)
    return [r.choice(["min", "max"]), "("] + expr(lo, m) + [","] + expr(m + 1, hi) + [")"]
print(len(ns))
for n in ns:
    print(n); print(*expr(1, n))
