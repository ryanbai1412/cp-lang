#!/usr/bin/env python3
"""egoi23p3 (Find the Box) cases: `gen.py SEED SIZE`, or `gen.py --max`.

"H W R C": the grid and the hidden box (R, C) != (0, 0); interactor.py sends only "H W".
--max: H = W = 50 with the box in the last row, the deepest a probe has to look."""
import random, sys
if sys.argv[1:] == ["--max"]:
    print(50, 50, 49, random.Random(0).randrange(50))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
top = max(2, min(50, S))
while True:
    h, w = r.randint(1, top), r.randint(1, top)
    if h * w >= 2: break
cell = r.randrange(1, h * w)
print(h, w, cell // w, cell % w)
