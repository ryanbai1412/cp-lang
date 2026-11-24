#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 500 tests of n = 100: one long single-0-gap block, alternating 1010...1, all ones,
    # and random strings
    g = random.Random(1)
    print(500)
    for i in range(500):
        s = ["1" + "01" * 49 + "1", "10" * 50, "1" * 100,
             "".join(g.choice("01") for _ in range(100))][i % 4]
        print(100)
        print(s[:100])
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(3, 18)
    print(n)
    print("".join(rng.choice("01") for _ in range(n)))
