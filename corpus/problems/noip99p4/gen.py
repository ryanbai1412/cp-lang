"""gen.py SEED SIZE | --max: NOIP 1999 P4. One line `N K` with N, K >= 1 and N + K <= 13. The
accepted C++ (an exhaustive search, the intended solution) runs past the 20 s reference budget on
(3, 9), (3, 10), (4, 8), (4, 9), (5, 7), (5, 8), (6, 6), (6, 7) and (7, 6), so those are never
drawn; --max is (8, 5), the slowest N + K = 13 pair it finishes (about 7 s)."""
import random, sys
SLOW = {(3, 9), (3, 10), (4, 8), (4, 9), (5, 7), (5, 8), (6, 6), (6, 7), (7, 6)}
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
pairs = [(n, k) for n in range(1, 13) for k in range(1, 14 - n) if n + k <= min(13, 2 + S)
         and (n, k) not in SLOW]
print(*((8, 5) if MAX else r.choice(pairs)))
