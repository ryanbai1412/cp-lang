"""gen.py SEED SIZE | --max: CCC 2018 S4. One weight n in 1..=1e9, its digit count random.
--max is n = 1e9."""
import random, sys
if sys.argv[1] == "--max":
    print(10**9)
else:
    r = random.Random(int(sys.argv[1]))
    print(r.randint(1, 10 ** r.randint(1, 7)))
