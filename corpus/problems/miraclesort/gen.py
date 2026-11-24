"""gen.py SEED SIZE | --max: Miracle Sort. N <= 1e6, then N numbers in [-2^63, 2^63 - 1], one
per line. The judge's data is already in order (the 6 MB limit can't hold 1e6 64-bit
numbers, and every accepted solution echoes its input), so the sequence is sorted, with
repeats and both extremes. --max is N = 1e6 spread over the full range."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1_000_000 if MAX else r.randint(1, min(1_000_000, 2 + 3 * S))
lo, hi = -(2 ** 63), 2 ** 63 - 1
pool = [lo, hi, 0, -1, 1] + [r.randint(lo, hi) for _ in range(4)]
xs = sorted(r.randint(lo, hi) if MAX or r.random() < 0.7 else r.choice(pool) for _ in range(n))
sys.stdout.write("\n".join(map(str, [n, *xs])) + "\n")
