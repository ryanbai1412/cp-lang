"""ioi22p2 generator: the one number n, 2 <= n <= 5000."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
print(5000 if big else r.randint(2, min(5000, 2 + size * size * r.choice([1, 10, 100]))))
