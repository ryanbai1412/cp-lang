"""gen.py SEED SIZE | --max: DMOPC '20 Contest 4 P1 (Missing Numbers). T <= 100 cases of
`N S`, 2 <= N <= 1e9, 0 <= S <= 1e18. S mostly sits near the full sum N(N+1)/2 minus a pair,
so answers are non-zero; some S are out of reach. --max is T = 100 with N = 1e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
t = 100 if MAX else r.randint(1, min(100, 1 + S))
out = [str(t)]
for i in range(t):
    n = 10**9 - i if MAX else r.randint(2, min(10**9, 2 + S * S))
    tot = n * (n + 1) // 2
    k = r.random()
    if k < 0.7:
        s = tot - r.randint(1, 2 * n)
    elif k < 0.85:
        s = r.randint(0, 10**18)
    else:
        s = max(0, tot - r.choice([1, 2, 3, n, n + 1, 2 * n - 1, 2 * n]))
    out.append(f"{n} {max(0, min(10**18, s))}")
sys.stdout.write("\n".join(out) + "\n")
