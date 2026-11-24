"""gen.py SEED SIZE | --max: CCC 2020 S5. n people with favourite burger types in 1..=5e5, often
repeated. --max is n = 1e6 over 5e5 types. n = 1 (answer 1) is left out: the accepted
C++ prints 0 there."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10 ** 6 if MAX else r.randint(2, 2 + 4 * S)
t = 5 * 10 ** 5 if MAX else r.randint(1, max(1, min(5 * 10 ** 5, r.choice([2, 5, n]))))
print(n)
print(*[r.randint(1, t) for _ in range(n)])
