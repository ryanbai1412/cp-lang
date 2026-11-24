"""noi02p2 hidden data: N and M (2 <= N + M <= 500), then the N + M + 1 roles in child order:
0 Star, 1 Moon, 2 the referee."""
import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "500"]
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
total = 500 if size >= 500 else rng.randint(2, max(2, size))
stars = rng.choice([0, total, rng.randint(0, total)])
roles = [0] * stars + [1] * (total - stars) + [2]
rng.shuffle(roles)
print(stars, total - stars)
print(*roles)
