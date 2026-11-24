"""sdoi17r2p6 (SDOI 2017 Text Correction) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

Case, then per case "n m", S and T with 3 <= n <= 1e6, 1 <= S_i, T_i <= m <= 1e6, following
the statement's table: n, m <= 6 allows Case <= 5e4 with sums <= 3e5, otherwise
Case <= 30 with sum n, sum m <= 1e6. T is random or periodic with noise over a small
alphabet; S is T cut in three and reordered (any of the six orders), sometimes with one
symbol changed, or random.
--max: four cases with sum n = sum m = 1e6: periodic texts answered by CBA, ACB and BAC,
and a large-alphabet near miss.
"""
import random
import sys

ORDERS = [(0, 1, 2), (0, 2, 1), (1, 0, 2), (1, 2, 0), (2, 0, 1), (2, 1, 0)]
MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))


def text(n, m, per, noise):
    base = [rng.randint(1, m) for _ in range(per)]
    return [base[i % per] if rng.random() >= noise else rng.randint(1, m) for i in range(n)]


def case(n, m, order=None, tweak=None, per=None):
    per = per or rng.choice([1, 2, 3, 5, n])
    T = text(n, m, per, rng.choice([0, 0.001, 0.05]))
    if order is None and rng.random() < 0.15:
        S = [rng.randint(1, m) for _ in range(n)]
    else:
        i = rng.randint(1, n - 2)
        j = rng.randint(i + 1, n - 1)
        p = [T[:i], T[i:j], T[j:]]
        S = [x for k in (order or rng.choice(ORDERS)) for x in p[k]]
        if tweak or (tweak is None and rng.random() < 0.3):
            S[rng.randrange(n)] = rng.randint(1, m)
    return f"{n} {m}\n{' '.join(map(str, S))}\n{' '.join(map(str, T))}"


out = []
if MAX:
    q = 250000
    out.append(case(q, 2, (2, 1, 0), False, 2))
    out.append(case(q, 2, (0, 2, 1), False, 3))
    out.append(case(q, 3, (1, 0, 2), False, 1))
    out.append(case(q, 10**6 - 7, (2, 1, 0), True, q))
else:
    size = max(3, int(sys.argv[2]))
    if size <= 6 or rng.random() < 0.2:
        cases, n_cap, budget = rng.randint(1, min(5 * 10**4, size * 10)), 6, 3 * 10**5
    else:
        cases, n_cap, budget = rng.randint(1, 30), min(10**6, size), 10**6
    left_n = left_m = budget
    for _ in range(cases):
        if left_n < 3 or left_m < 1:
            break
        n = rng.randint(3, max(3, min(n_cap, left_n)))
        m = rng.randint(1, min(n_cap, left_m, rng.choice([2, 3, n_cap])))
        left_n -= n
        left_m -= m
        out.append(case(n, m))
print(len(out))
print("\n".join(out))
