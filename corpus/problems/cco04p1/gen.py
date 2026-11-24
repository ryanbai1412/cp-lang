"""gen.py SEED SIZE | --max: CCO 2004 P1. k distinct letters with scores <= 26 and counts making
7 tiles, then n <= 1e5 lowercase words. Small cases use few letters so words often fit. --max is
n = 1e5 words of up to 10 letters drawn from k = 7 distinct letters, one tile each (the statement
has 1 <= k <= 7 and the counts sum to 7)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
abc = "abcdefghijklmnopqrstuvwxyz"[:26 if MAX else r.choice([3, 5, 26])]
counts = {}
if MAX: counts = {c: 1 for c in r.sample(abc, 7)}
else:
    for _ in range(7): c = r.choice(abc); counts[c] = counts.get(c, 0) + 1
n = 100_000 if MAX else r.randint(0, 1 + 3 * S)
out = [str(len(counts))] + [f"{c} {r.randint(0, 26)} {x}" for c, x in counts.items()]
pool = "".join(counts) if MAX or r.random() >= .5 else abc
out += [str(n)] + ["".join(r.choice(pool) for _ in range(r.randint(1, 10 if MAX else 8))) for _ in range(n)]
sys.stdout.write("\n".join(out) + "\n")
