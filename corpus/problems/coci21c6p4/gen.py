"""gen.py SEED SIZE | --max: COCI 2021/2022 Round 6 #4 (Palindromi). n <= 1e5 characters 0/1 and
n - 1 concatenations of two different current strings, each named by any of its characters.
Small cases mix random, all-zero and periodic strings and random, chain or balanced merges.
--max is n = 1e5 with long zero runs broken by ones and random merges."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, 1 + 2 * S)
kind = 3 if MAX else r.randrange(4)
if kind == 0:
    s = "".join(r.choice("01") for _ in range(n))
elif kind == 1:
    s = "0" * n
elif kind == 2:
    w = "".join(r.choice("01") for _ in range(r.randint(1, 4)))
    s = (w * n)[:n]
else:
    s = "".join("1" if r.random() < 0.01 else "0" for _ in range(n))
groups = [[i] for i in range(1, n + 1)]
order = r.randrange(3) if not MAX else 0
out = []
while len(groups) > 1:
    if order == 1:
        i, j = 0, 1
    else:
        i, j = r.sample(range(len(groups)), 2)
    a, b = groups[i], groups[j]
    out.append(f"{r.choice(a)} {r.choice(b)}\n")
    a.extend(b)
    groups[j] = groups[-1]
    groups.pop()
    if order == 1 and len(groups) > 1:
        groups[0], groups[-1] = groups[-1], groups[0]
        groups[1], groups[-1] = groups[-1], groups[1]
sys.stdout.write(f"{n}\n{s}\n" + "".join(out))
