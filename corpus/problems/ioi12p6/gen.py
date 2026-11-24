#!/usr/bin/env python3
"""gen.py seed size | --max: IOI 2012 Jousting Tournament. n <= 1e5 knights, c < n rounds
"s e" (0 <= s < e < the knights left), the late knight's rank r and the others' ranks, a
permutation of 0..n-1 without r. --max: n = 1e5, c = n - 1 rounds, most of them small merges
at the front (a deep chain of rounds), all of them pairs so c reaches n - 1."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 100_000 if MAX else max(2, min(100_000, int(sys.argv[2])))
rank = r.randrange(n)
others = [x for x in range(n) if x != rank]; r.shuffle(others)
rounds, left, c = [], n, n - 1 if MAX else r.randint(1, n - 1)
while left > 1 and len(rounds) < c:
    if MAX:
        s = r.randrange(min(3, left - 1)) if r.random() < 0.8 else r.randrange(left - 1); e = s + 1
    else:
        s = r.randrange(left - 1); e = r.randint(s + 1, min(left - 1, s + 1 + r.randint(0, 50)))
    rounds.append(f"{s} {e}"); left -= e - s
print(n, len(rounds), rank)
print("\n".join(map(str, others)))
print("\n".join(rounds))
