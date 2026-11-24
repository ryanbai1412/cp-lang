"""gen.py SEED SIZE | --max: NOIP 2009 P1. Three uppercase strings of length 1..=100, the first
two of equal length (encrypted, original). Cases mix a consistent full key (all 26 letters), a
missing letter and a contradiction. --max is a consistent 100-letter pair with all 26 letters."""
import random, sys
A = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
key = list(A)
r.shuffle(key)
kind = 0 if MAX else r.randint(0, 3)
n = 100 if MAX else r.randint(1, 100) if kind == 3 else r.randint(26, min(100, 26 + 10 * S))
orig = list(A) + [r.choice(A) for _ in range(n - 26)] if kind != 3 else [r.choice(A) for _ in range(n)]
r.shuffle(orig)
if kind == 1:
    gone = r.choice(A)
    orig = [c if c != gone else r.choice([x for x in A if x != gone]) for c in orig]
enc = [key[ord(c) - 65] for c in orig]
if kind == 2 and n > 1:
    enc[r.randrange(n)] = r.choice(A)
m = 100 if MAX else r.randint(1, min(100, 1 + 10 * S))
print("".join(enc))
print("".join(orig))
print("".join(r.choice(A) for _ in range(m)))
