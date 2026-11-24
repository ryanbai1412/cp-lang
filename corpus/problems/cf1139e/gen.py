"""gen.py SEED SIZE | --max: Codeforces 1139E (Maximize Mex). n <= 5000 students with potential
0..4999 in m <= n clubs; d <= n of them leave, one a day. Small cases use small potentials so the
mex moves. --max is n = m = d = 5000 with potential i mod 2500, so the mex starts near 2500 and
every day needs augmenting paths."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 5000 if MAX else r.randint(1, 1 + 2 * S)
m = 5000 if MAX else r.randint(1, n)
if MAX:
    p = [i % 2500 for i in range(n)]
    c = [r.randint(1, m) for _ in range(n)]
else:
    p = [r.randint(0, min(4999, r.randint(0, S + 1))) for _ in range(n)]
    c = [r.randint(1, m) for _ in range(n)]
d = n if MAX else r.randint(1, n)
k = r.sample(range(1, n + 1), d)
sys.stdout.write(f"{n} {m}\n{' '.join(map(str, p))}\n{' '.join(map(str, c))}\n{d}\n" + "".join(f"{x}\n" for x in k))
