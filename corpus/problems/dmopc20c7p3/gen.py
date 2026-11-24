"""gen.py SEED SIZE | --max: DMOPC '20 Contest 7 P3 (Senpai and Art). T cases of N, L, R with
R - L + 1 >= ceil(N / 2), sum N <= 1e6, A_i <= 1e9. A is mostly a sum of random strokes
(some longer than R), sometimes perturbed so it is impossible. --max is one N = 1e6 case,
L = N / 4, built from 1e6 random strokes so the queue carries many overlong ones."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))


def case(n, strokes, small):
    lo = r.randint(1, n - (n + 1) // 2 + 1)
    hi = r.randint(lo + (n + 1) // 2 - 1, n)
    diff = [0] * (n + 2)
    for _ in range(strokes):
        ln = r.randint(lo, n)
        s = r.randint(1, n - ln + 1)
        c = r.randint(1, 3 if small else 1000)
        diff[s] += c
        diff[s + ln] -= c
    a, run = [], 0
    for i in range(1, n + 1):
        run += diff[i]
        a.append(run)
    if not MAX and r.random() < 0.3:
        a[r.randrange(n)] = r.randint(0, 3 if small else 10**9)
    return f"{n} {lo} {hi}\n{' '.join(map(str, a))}"


if MAX:
    n = 10**6
    lo, hi = n // 4, n // 4 + n // 2
    diff = [0] * (n + 2)
    for _ in range(n):
        ln = r.choice([r.randint(lo, hi), r.randint(hi + 1, n)])
        s = r.randint(1, n - ln + 1)
        diff[s] += 1
        diff[s + ln] -= 1
    a, run = [], 0
    for i in range(1, n + 1):
        run += diff[i]
        a.append(run)
    sys.stdout.write(f"1\n{n} {lo} {hi}\n{' '.join(map(str, a))}\n")
    sys.exit()
budget = min(10**6, int(sys.argv[2]))
small = budget <= 8
out = []
while budget > 0 and len(out) < 10**6:
    n = r.randint(1, min(budget, 8 if small else budget))
    budget -= n
    out.append(case(n, r.randint(0, 4 if small else 2 * n), small))
    if r.random() < 0.3:
        break
sys.stdout.write(f"{len(out)}\n" + "\n".join(out) + "\n")
