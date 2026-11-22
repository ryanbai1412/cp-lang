import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
h, w = r.randint(2, 2 * S + 1), r.randint(2, 2 * S + 1)
cells = [(a, b) for a in range(1, h + 1) for b in range(1, w + 1) if (a, b) not in ((1, 1), (h, w))]
n = r.randint(1, min(len(cells), 3 * S))
print(h, w, n)
for a, b in r.sample(cells, n): print(a, b)
