import sys, random
if sys.argv[1] == "--max": sys.argv[1:] = ["0", "81"]
# usage: gen.py seed size; size >= 81 is a 9 x 9 grid with 20 numbered cells, the most the
# judge's data has (its only accepted C++ is exponential in the numbered cells)
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 9 if S >= 81 else r.randint(1, 9)
cells = [(x, y) for x in range(1, n + 1) for y in range(1, n + 1)]
k = min(20, len(cells)) if S >= 81 else r.randint(0, min(20, len(cells), max(1, S)))
picked = r.sample(cells, k)
hi = 30 if S >= 81 or r.random() < 0.5 else r.randint(1, 30)
print(n)
for x, y in picked: print(x, y, hi if S >= 81 and r.random() < 0.5 else r.randint(1, hi))
print(0, 0, 0)  # the statement ends the list with a line 0 0 0
