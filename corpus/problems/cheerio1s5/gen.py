"""Cheerio 1 S5 inputs: `gen.py seed size` (up to `size` coins) or `gen.py --max` (400 coins,
mostly heads, K = number of heads)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 400 if MAX else r.randint(1, max(1, min(400, int(sys.argv[2]))))
p = 0.9 if MAX else r.random()
s = [("H" if r.random() < p else "T") for _ in range(n)]
if "H" not in s: s[r.randrange(n)] = "H"
h = s.count("H")
print("".join(s))
print(h if MAX else r.randint(1, h))
