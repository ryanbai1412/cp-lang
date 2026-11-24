#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test with n = 1000 strings and total length 4000, nearly all 'a' so every
    # s + a vs a + s comparison runs the whole string; a few 'b' tails break the ties
    g = random.Random(1)
    words = ["aaa" + ("b" if g.random() < 0.01 else "a") for _ in range(1000)]
    print(1)
    print(1000)
    print(*words)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 10)
    print(n)
    words = ["".join(rng.choice("abc") for _ in range(rng.randint(1, 4))) for _ in range(n)]
    print(*words)
