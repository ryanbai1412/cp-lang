"""gen.py SEED SIZE | --max: Riolku's Mock CCC S2. N in 1..=1500 values a_i in 1..=1e9. Small
sizes draw from small values or few powers of two so products repeat; --max is N = 1500 values
from {2^i 3^j} up to 1e9, the many-equal-products case."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1500 if MAX else r.randint(1, min(1500, 1 + 3 * S))
smooth = [2**i * 3**j for i in range(30) for j in range(19) if 2**i * 3**j <= 10**9]
kind = 0 if MAX else r.randint(0, 2)
pick = [lambda: r.choice(smooth[:40]), lambda: r.randint(1, 1 + S), lambda: r.randint(1, 10**9)][kind]
print(n)
print(*(pick() for _ in range(n)))
