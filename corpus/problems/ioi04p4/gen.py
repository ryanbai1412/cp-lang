import sys, random
MAX = sys.argv[1] == "--max"  # the largest slab and the most plates
r = random.Random(1 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
w, h, n = (600, 600, 200) if MAX else (r.randint(1, 2 * S + 1), r.randint(1, 2 * S + 1), r.randint(1, 5)); print(w, h); print(n)
for _ in range(n): print(r.randint(1, w), r.randint(1, h))
