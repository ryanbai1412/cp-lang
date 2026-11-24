#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SDOI 2017 Sequence Counting. `n m p` with n <= 1e9, m <= 2e7,
p <= 100; SIZE caps m. --max: n = 1e9, m = 2e7, p = 100."""
import random, sys
if sys.argv[1:] == ["--max"]:
    print(10**9, 2 * 10**7, 100)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 2 * 10**7)
print(r.choice([r.randint(1, 10), r.randint(1, 10**9)]), r.randint(1, size), r.randint(1, 100))
