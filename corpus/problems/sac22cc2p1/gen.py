"""gen.py SEED SIZE | --max: SAC '22 CC2 P1. A <= 100 assessments (day 1..=30, a name of 1..=15
letters or _), then K <= 30 strictly increasing check days in 1..=30. --max is A = 100, K = 30."""
import random, string, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
a = 100 if MAX else r.randint(1, min(100, 1 + 3 * S))
k = 30 if MAX else r.randint(1, min(30, 1 + S))
ch = string.ascii_letters + "_"
print(a, k)
for _ in range(a):
    print(r.randint(1, 30), "".join(r.choice(ch) for _ in range(15 if MAX else r.randint(1, 15))))
print(*sorted(r.sample(range(1, 31), k)), sep="\n")
