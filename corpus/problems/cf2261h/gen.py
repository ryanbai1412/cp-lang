#!/usr/bin/env python3
# First-run input only; interactor.py plays Lord Verity and the second run.
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.choice([1, 2, 5, 1024]); print("first"); print(t)
for _ in range(t):
    n, h = (1, 2) if r.random() < 0.1 else (2030, 11)
    print(n, h); print("".join(r.choice("01") for _ in range(n)))
