"""gen.py SEED SIZE | --max: DMOPC '21 Contest 5 P5 (Permutations & Primes). One N in
1..=100. --max is N = 100."""
import random, sys
if sys.argv[1] == "--max":
    print(100)
    sys.exit()
r = random.Random(int(sys.argv[1]))
print(r.randint(1, max(1, min(100, int(sys.argv[2])))))
