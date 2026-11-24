"""gen.py SEED SIZE | --max: Fast Factorial Calculator. N <= 1000 queries of n in 0..=2^63, asking
for n! mod 2^32; small n, n near the modulus and huge n are all mixed in. --max is N = 1000
with n spread just below 2^32 - 5 (the slowest direct products) and a few huge."""
import random, sys
P = 2**32 - 5
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
N = 1000 if MAX else r.randint(0, min(1000, int(sys.argv[2])))


def pick():
    if MAX:
        return r.choice([P - 1 - r.randint(0, 10**6), r.randint(P // 2, P - 1), r.randint(P, 2**63)])
    k = r.random()
    if k < 0.3:
        return r.randint(0, 40)
    if k < 0.6:
        return r.randint(0, 10**9)
    if k < 0.8:
        return r.randint(P - 10**5, P + 10**5)
    return r.choice([2**63, 2**63 - 1, r.randint(0, 2**63)])


sys.stdout.write("\n".join([str(N)] + [str(pick()) for _ in range(N)]) + "\n")
