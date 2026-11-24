import sys, random
# seed size: k in 1..9, w <= 30000 scaled by size; --max: k = 9, w = 30000 (every C(511, d) term)
mx = sys.argv[1] == "--max"
if mx:
    print(9, 30000); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, 9), r.randint(1, min(30000, 4 * S)))
