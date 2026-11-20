import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# a strip of quadrilateral pens; each either shares its first wall with the pen before or stands alone
p = r.randint(1, S + 1); print(p); nxt = 1; wall = None
for i in range(p):
    if wall is None:  # pens stay connected: the accepted original sums a forest otherwise
        wall = ((nxt, nxt + 1), r.randint(1, 20)); nxt += 2
    (a, b), cost = wall; c, d = nxt, nxt + 1; nxt += 2
    costs = [cost] + [r.randint(1, 20) for _ in range(3)]
    print(4); print(a, b, d, c); print(*costs)
    wall = ((d, c), costs[2])
