#!/usr/bin/env python3
# The statement allows only n = 4 and n = 250000.
import random, sys
print(random.Random(int(sys.argv[1])).choice([4, 250000]))
