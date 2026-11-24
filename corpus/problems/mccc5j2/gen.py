"""gen.py SEED SIZE | --max: Mock CCC '23 C1 J2. Bedtime 13 <= S <= 23 and wake-up 0 <= W <= 12
on two lines. Any size draws from the full range, half the time from the healthy window's
edges; --max is S = 23, W = 12."""
import random, sys
if sys.argv[1] == "--max":
    print(23, 12, sep="\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
if r.random() < 0.5:
    print(r.choice([19, 20, 21, 22, 23]), r.choice([5, 6, 7, 8, 9, 10]), sep="\n")
else:
    print(r.randint(13, 23), r.randint(0, 12), sep="\n")
