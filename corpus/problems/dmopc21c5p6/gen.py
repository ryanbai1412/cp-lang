"""gen.py SEED SIZE | --max: DMOPC '21 Contest 5 P6 (Permutations & Primes). One integer
1 <= N <= 1e6 (at most SIZE); --max is N = 1e6."""
import random, sys
if sys.argv[1] == "--max":
    print(10**6)
else:
    print(random.Random(int(sys.argv[1])).randint(1, max(1, min(10**6, int(sys.argv[2])))))
