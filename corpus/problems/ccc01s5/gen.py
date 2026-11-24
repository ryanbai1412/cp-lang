"""gen.py SEED SIZE | --max: CCC 2001 S5. m * n <= 40 and n pairs of words over a small alphabet,
some built to agree. --max is m = 20, n = 2 with 20-letter words."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 2 if MAX else r.randint(1, min(4, 1 + S))
m = 40 // n if MAX else r.randint(1, min(8, 40 // n))
L = 20 if MAX else r.randint(1, 4)
abc = "ab" if MAX or r.random() < 0.7 else "abc"
w = lambda: "".join(r.choice(abc) for _ in range(r.randint(1, L)))
a = [w() for _ in range(n)]; b = [w() for _ in range(n)]
if not MAX and r.random() < 0.5:  # plant a solution: split one concatenation two ways
    k = r.randint(1, n); s = "".join(r.choice(abc) for _ in range(r.randint(1, L)))
    cut = r.randint(0, len(s)); a[k - 1] = s; b[k - 1] = s[:cut] + s[cut:] if cut else s
print(m); print(n)
for x in a: print(x)
for x in b: print(x)
