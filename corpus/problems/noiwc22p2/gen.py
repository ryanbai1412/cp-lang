"""gen.py SEED SIZE | --max: WC 2022 P2. A permutation of n in 1..=5e5 and m in 1..=5e5 queries
1 <= l <= r <= n. Small sizes mix random and near-sorted (|a_i - i| <= 10) permutations and
short or long ranges; --max is n = m = 5e5, a random permutation and random wide ranges."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 500_000 if MAX else r.randint(1, min(500_000, 1 + 4 * S))
m = 500_000 if MAX else r.randint(1, min(500_000, 1 + 4 * S))
a = list(range(1, n + 1))
if MAX or r.random() < 0.6:
    r.shuffle(a)
else:
    for i in range(n - 1):
        j = r.randint(i, min(n - 1, i + 10))
        if abs(a[j] - (j + 1)) < 10 and abs(a[i] - (i + 1)) < 10 and abs(a[i] - (j + 1)) <= 10 and abs(a[j] - (i + 1)) <= 10:
            a[i], a[j] = a[j], a[i]
out = [f"{n} {m}", " ".join(map(str, a))]
for _ in range(m):
    if MAX or r.random() < 0.5:
        l, rr = sorted((r.randint(1, n), r.randint(1, n)))
    else:
        l = r.randint(1, n)
        rr = min(n, l + r.randint(0, 5))
    out.append(f"{l} {rr}")
sys.stdout.write("\n".join(out) + "\n")
