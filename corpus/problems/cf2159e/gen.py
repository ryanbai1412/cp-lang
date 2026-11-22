import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
M = 10**9 + 7
a, b, c = (r.choice([1, 2, M - 1, r.randint(1, M - 1)]) for _ in range(3))
q = r.randint(1, 3 * S)
nmax = r.choice([5, 50, 700 * S])
inv = [0, 1]
for i in range(2, 2 * nmax + 1):
    inv.append((M - (M // i) * inv[M % i] % M) % M)
ic = pow(c, M - 2, M)


def answer(n, k):
    f = [pow(c, n, M)]
    for z in range(1, k + 1):
        acc = f[z - 1] * b % M * (n - z + 1)
        if z >= 2:
            acc += f[z - 2] * a % M * (2 * n - z + 2)
        f.append(acc % M * inv[z] % M * ic % M)
    return sum(f) % M


print(a, b, c)
print(q)
last = 0
for _ in range(q):
    n = r.randint(0, nmax)
    k = r.choice([0, 2 * n, r.randint(0, 2 * n)])
    print(n ^ last, k ^ last)
    last = answer(n, k)
