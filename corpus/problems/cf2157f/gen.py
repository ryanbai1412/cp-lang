#!/usr/bin/env python3
# The statement allows only n = 4 and n = 250000.
import random, sys
# --max is the large test, n = 250000.
if sys.argv[1] == "--max":
    print(250000)
else:
    print(random.Random(int(sys.argv[1])).choice([4, 250000]))
