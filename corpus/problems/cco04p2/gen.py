#!/usr/bin/env python3
# One random CCO 2004 P2 input: gen.py <seed> [size]. The typed generator
# cannot make `a-b` words (backlog item S8), so this stands in for it.
import random, sys
if sys.argv[1:] == ["--max"]:
    # the statement bounds s <= 1000 but not the test count or the score values: 100 tests of
    # s = 1000, alternating antichains (every score its own game, the most rows to search)
    # with long chains, scores up to 1e6 in either order
    g = random.Random(1)
    print(100)
    for t in range(100):
        if t % 2:
            sc = [(i, 10**6 - i) for i in range(1000)]
        else:
            sc = sorted((g.randint(0, 10**6), g.randint(0, 10**6)) for _ in range(1000))
        g.shuffle(sc)
        print(1000)
        print(" ".join(f"{x}-{y}" if g.random() < 0.5 else f"{y}-{x}" for x, y in sc))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 6
tests = rng.randint(1, 3)
print(tests)
for _ in range(tests):
    n = rng.randint(1, size)
    print(n)
    print(" ".join(f"{rng.randint(1, size)}-{rng.randint(1, size)}" for _ in range(n)))
