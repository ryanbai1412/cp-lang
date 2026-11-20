import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
h = r.randint(2, S + 1); w = r.randint(2, S + 1)
print(h, w)
g = [['#' if r.randint(0, 4) == 0 else '.' for _ in range(w)] for _ in range(h)]
g[0][0] = g[-1][-1] = '.'
for row in g: print(''.join(row))
