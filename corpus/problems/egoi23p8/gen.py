#!/usr/bin/env python3
"""egoi23p8 (Guessing Game) cases: `gen.py SEED SIZE`, or `gen.py --max`.

"N SEED": the street and the seed interactor.py draws Emma's house, the order of the other houses
and Emma's label from. --max: N = 1e5, the statement's street."""
import random, sys
if sys.argv[1:] == ["--max"]:
    print(100000, 0)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(13, max(13, min(100000, 13 + S))), int(sys.argv[1]))
