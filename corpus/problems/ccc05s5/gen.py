"""gen.py SEED SIZE | --max: CCC 2005 S5. t game scores in 0..=1e9, in order, often repeated.
--max is t = 1e5 scores up to 1e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
t = 100000 if MAX else r.randint(1, 2 + 4 * S)
top = 10 ** 9 if MAX else r.choice([3, 10, 10 ** 9])
print(t)
for _ in range(t): print(r.randint(0, top))
