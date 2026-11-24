import sys, random
MAX = sys.argv[1] == "--max"  # 100 pens, costs up to 5000
r = random.Random(0 if MAX else int(sys.argv[1])); S = 99 if MAX else int(sys.argv[2]); C = 5000 if MAX else 20
# a strip of quadrilateral pens; each either shares its first wall with the pen before or stands alone
p = S + 1 if MAX else r.randint(1, min(100, S + 1)); print(p); nxt = 1; wall = None
for i in range(p):
    if wall is None:  # pens stay connected: the accepted original sums a forest otherwise
        wall = ((nxt, nxt + 1), r.randint(1, C)); nxt += 2
    (a, b), cost = wall; c, d = nxt, nxt + 1; nxt += 2
    costs = [cost] + [r.randint(1, C) for _ in range(3)]
    print(4); print(a, b, d, c); print(*costs)
    wall = ((d, c), costs[2])
