"""gen.py SEED SIZE | --max: Mock CCC '23 C1 J1. u <= 30 and t <= 10 on two lines. Any size
draws from the full range; --max is u = 30, t = 10."""
import random, sys
if sys.argv[1] == "--max":
    print(30, 10, sep="\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
print(r.randint(0, 30), r.randint(0, 10), sep="\n")
