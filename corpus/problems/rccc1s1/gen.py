"""gen.py SEED SIZE | --max: Riolku's Mock CCC S1. N, C, V in 1..=1e5 and a lowercase word of N
letters. Small sizes keep C and V near the run lengths so both answers occur; --max is
N = C = V = 1e5 over all 26 letters."""
import random, string, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, min(100_000, 1 + 4 * S))
c = 100_000 if MAX else r.randint(1, min(100_000, 2 + S))
v = 100_000 if MAX else r.randint(1, min(100_000, 2 + S))
al = r.choice(["aeiouy", "bcdy", "aeiou", "bcdfg", string.ascii_lowercase, "aybz"]) if not MAX else string.ascii_lowercase
print(n, c, v)
print("".join(r.choice(al) for _ in range(n)))
