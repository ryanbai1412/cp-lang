import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0)
    h = w = 100000
    cells = set()
    while len(cells) < 3000:
        c = (r.randint(1, h), r.randint(1, w))
        if c not in ((1, 1), (h, w)): cells.add(c)
    print(h, w, 3000)
    for a, b in sorted(cells, key=lambda _: r.random()): print(a, b)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
h, w = r.randint(2, 2 * S + 1), r.randint(2, 2 * S + 1)
cells = [(a, b) for a in range(1, h + 1) for b in range(1, w + 1) if (a, b) not in ((1, 1), (h, w))]
n = r.randint(1, min(len(cells), 3 * S))
print(h, w, n)
for a, b in r.sample(cells, n): print(a, b)
