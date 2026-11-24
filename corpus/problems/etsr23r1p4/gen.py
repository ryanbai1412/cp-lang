# etsr23r1p4 generator: python3 gen.py SEED [SIZE] | --max. Range add, floor division, min, sum.
import random, sys

MAX = sys.argv[1] == "--max"
r = random.Random(1 if MAX else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 10
n = 200000 if MAX else r.randint(1, max(1, size))
q = 200000 if MAX else r.randint(1, max(1, 2 * size))
V = 10**9 if MAX else r.choice([5, 100, 10**9])
a = [r.randint(-V, V) for _ in range(n)]
out = [f"{n} {q}", " ".join(map(str, a))]
for _ in range(q):
    op = r.randint(1, 4)
    l = r.randint(1, n)
    rr = r.randint(l, n)
    if op == 1:
        # random-sign adds drift about sqrt(q) * 1e4, far inside 32 bits
        out.append(f"{op} {l} {rr} {r.randint(-10**4, 10**4)}")
    elif op == 2:
        out.append(f"{op} {l} {rr} {r.choice([1, 2, 3, r.randint(1, 10**9)])}")
    else:
        out.append(f"{op} {l} {rr}")
print("\n".join(out))
