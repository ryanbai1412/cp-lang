"""gen.py SEED SIZE | --max: IATI 2021 senior day 1 task 1 (Heaps): K Q, then Q tests of N
heaps `B S`. A seed picks one of the statement's subtask shapes (K = 0 with B = 0, B <= 1 or
B <= 300; K = 1 with B <= 5; K, B up to 20, 100, 300 or 3000) scaled by SIZE; half the tests
are heaps in equal pairs (a lost position) plus maybe one more. --max is K = 3000, Q = 10,
N = 10 000, B up to 3000 and S up to 10^7."""
import random, sys

if sys.argv[1] == "--max":
    r = random.Random(2021)
    k, kb, q, nmax, smax = 3000, 3000, 10, 10_000, 10 ** 7
else:
    r = random.Random(int(sys.argv[1]))
    size = int(sys.argv[2])
    shape = r.randrange(8)
    k, kb = [(0, 0), (0, 1), (0, 300), (1, 5), (20, 20), (100, 100), (300, 300), (3000, 3000)][shape]
    k = r.randint(0, k) if shape >= 4 else k
    kb = min(kb, 10 * size)
    q, nmax, smax = r.randint(1, 10), min(10_000, 4 * size), min(10 ** 7, 50 * size * (k + 1))


def heap():
    b = r.randint(0, kb)
    if kb == 1 and b == 1:
        return (1, 0)
    return (b, r.randint(0, smax) if r.random() < 0.5 else r.randint(0, max(0, k * b + 3)))


print(k, q)
for i in range(q):
    n = nmax if sys.argv[1] == "--max" else r.randint(1, nmax)
    if i % 2:
        hs = [heap() for _ in range(n // 2)]
        hs += hs + ([heap()] if n % 2 and r.random() < 0.5 else [])
        hs += [heap() for _ in range(n - len(hs))]
        r.shuffle(hs)
    else:
        hs = [heap() for _ in range(n)]
    if sys.argv[1] == "--max" and i == 0:
        hs[0] = (3000, 10 ** 7)
    print(len(hs))
    print("\n".join(f"{b} {s}" for b, s in hs))
