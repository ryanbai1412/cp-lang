# etsr23r4p2 generator: python3 gen.py SEED [SIZE] | --max. Range c^a_i raisings and range sums mod p.
import random, sys

MAX = sys.argv[1] == "--max"
r = random.Random(1 if MAX else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 50000 if MAX else r.randint(1, max(1, size))
q = 50000 if MAX else r.randint(1, max(1, 2 * size))


def phi(x):
    res, f = x, 2
    while f * f <= x:
        if x % f == 0:
            while x % f == 0:
                x //= f
            res -= res // f
        f += 1
    return res - res // x if x > 1 else res


def chain(x):
    k = 1
    while x > 1:
        x, k = phi(x), k + 1
    return k


# The accepted C++ caches phi below 1e7 and keeps 11 tower levels, so the judge's p are
# below 1e7 with a totient chain of at most 10 moduli.
def modulus(hi):
    while True:
        p = r.randint(1, hi)
        if chain(p) <= 10:
            return p


p = modulus(10**7) if MAX else r.choice([1, 2, 6, 127, modulus(1000), modulus(10**7)])
# the accepted C++ loops forever on c = 1, so the judge's tests have c >= 2
c = r.randint(2, 10**8) if MAX else r.choice([2, 3, r.randint(2, 50), r.randint(2, 10**8)])
a = [r.randint(0, p - 1) for _ in range(n)]
out = [f"{n} {q} {p} {c}", " ".join(map(str, a))]
for _ in range(q):
    l = r.randint(1, n)
    out.append(f"{r.randint(0, 1)} {l} {r.randint(l, n)}")
print("\n".join(out))
