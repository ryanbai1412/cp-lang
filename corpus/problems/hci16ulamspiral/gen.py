"""gen.py SEED SIZE | --max: HCI '16 Ulam Spiral. One integer 1 <= N <= 31 (at most SIZE);
--max is N = 31."""
import random, sys
if sys.argv[1] == "--max":
    print(31)
else:
    print(random.Random(int(sys.argv[1])).randint(1, max(1, min(31, int(sys.argv[2])))))
