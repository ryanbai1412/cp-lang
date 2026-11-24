"""gen.py SEED SIZE | --max: CCC 2023 S2. n mountain heights in 0..=1e5. --max is n = 5000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 5000 if MAX else r.randint(1, 1 + 4 * S)
top = 10 ** 5 if MAX else r.choice([2, 10, 10 ** 5])
print(n)
print(*[r.randint(0, top) for _ in range(n)])
