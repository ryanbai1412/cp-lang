"""ioi99p6 generator: U V C (columns, rows, limit), then V rows of U heights. Heights stay in a
narrow band around a slow slope so wide flat rectangles exist. --max: U = V = 700, C = 10."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
u = 700 if big else r.randint(1, max(1, size + 1))
v = 700 if big else r.randint(1, max(1, size + 1))
c = 10 if big else r.randint(0, 10)
spread = r.choice([3, 12, 30000]) if not big else 12
print(u, v, c)
for y in range(v):
    print(*[max(-30000, min(30000, (x + y) // 50 + r.randint(0, spread))) for x in range(u)])
