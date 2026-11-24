"""Cheerio 1 S2 inputs: `gen.py seed size` (N up to `size`) or `gen.py --max` (N = 1e5,
A = B = 1e4)."""
import sys, random
if sys.argv[1:] == ["--max"]:
    print(100000); print(10000); print(10000); sys.exit()
r = random.Random(int(sys.argv[1]))
print(r.randint(1, max(1, int(sys.argv[2])))); print(r.randint(1, 10**4)); print(r.randint(1, 10**4))
