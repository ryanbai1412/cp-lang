"""gen.py SEED SIZE | --max: DMOPC '21 Contest 5 P5 hard (maximum primeness permutation).
1 <= N <= 1e5, N drawn up to SIZE (capped at 1e5) with even and odd N alike.
--max is N = 1e5."""
import random, sys
if sys.argv[1] == "--max":
    print(10**5)
    sys.exit()
r = random.Random(int(sys.argv[1]))
print(r.randint(max(1, min(10**5, int(sys.argv[2])) // 2), min(10**5, max(1, int(sys.argv[2])))))
