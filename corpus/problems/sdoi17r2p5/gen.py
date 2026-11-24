"""sdoi17r2p5 (SDOI 2017 Forgotten Set) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"n p" then f(1..n) mod p, 1 <= n < 2^18, p a prime with 1e6 <= p < 2^30. The data comes
from a hidden nonempty set S with elements <= n, so a set fitting f always exists: f is its
partition counts mod p. S is random, small-element (s_m <= 40), or {1..m} (m = s_m).
--max: n = 2^18 - 1 and p = 1e9 + 7 with 300 random elements, one of them 1.
"""
import random
import sys

from itertools import accumulate

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))


def prime(x):
    if x < 2:
        return False
    for q in (2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37):
        if x % q == 0:
            return x == q
    d, r = x - 1, 0
    while d % 2 == 0:
        d //= 2
        r += 1
    for a in (2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37):
        y = pow(a, d, x)
        if y in (1, x - 1):
            continue
        for _ in range(r - 1):
            y = y * y % x
            if y == x - 1:
                break
        else:
            return False
    return True


if MAX:
    n, p = 2**18 - 1, 10**9 + 7
    S = sorted({1} | {rng.randint(2, n) for _ in range(299)})
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(1, min(2**18 - 1, size))
    p = rng.choice([1000003, 998244353, 1000000007, None])
    while p is None or not prime(p):
        p = rng.randint(10**6, 2**30 - 1)
    kind = rng.choice(["random", "small", "prefix"])
    if kind == "prefix":
        S = list(range(1, rng.randint(1, min(n, 2000)) + 1))
    else:
        top = min(n, 40) if kind == "small" else n
        S = sorted({rng.randint(1, top) for _ in range(rng.randint(1, min(top, 300)))})
f = [1] + [0] * n
for k, s in enumerate(S):
    for r in range(s):
        f[r::s] = accumulate(f[r::s])
    if k % 16 == 15 or k + 1 == len(S):
        f = [x % p for x in f]
print(n, p)
print(" ".join(map(str, f[1:])))
