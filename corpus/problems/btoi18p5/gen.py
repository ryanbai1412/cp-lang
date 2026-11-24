"""gen.py SEED SIZE | --max: BOI 2018 Genetics. N sequences of length M (3 <= N, M <= 4100) over
ACGT (sometimes only AC) with exactly one, the villain, at distance K from all the others. The
villain is random; each clone is the villain with exactly K positions changed, and clones that
happen to sit at distance K from all others are rerolled. --max is N = M = 4100 over ACGT."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 4_100 if MAX else r.randint(3, min(4_100, 3 + S))
m = 4_100 if MAX else r.randint(3, min(4_100, 3 + S))
abc = "ACGT" if MAX or r.random() < 0.6 else "AC"
k = r.randint(1, m) if not MAX else m // 4
v = [r.choice(abc) for _ in range(m)]


def clone():
    c = v[:]
    for j in r.sample(range(m), k):
        c[j] = r.choice([x for x in abc if x != v[j]])
    return c


def dist(a, b):
    return sum(x != y for x, y in zip(a, b))


while True:
    rows = [clone() for _ in range(n - 1)]
    at = r.randint(0, n - 1)
    rows.insert(at, v)
    # every clone must differ from some other row in other than K places; small cases check it
    if MAX or all(any(dist(rows[i], rows[j]) != k for j in range(n) if j != i) for i in range(n) if i != at):
        break
print(n, m, k)
print("\n".join("".join(x) for x in rows))
