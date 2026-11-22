import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
w = r.randint(1, 2 * S + 1); h = r.randint(1, 2 * S + 1); n = r.randint(1, 5); print(w, h); print(n)
for _ in range(n): print(r.randint(1, w), r.randint(1, h))
