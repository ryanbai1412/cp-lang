import sys, random
# seed size: n, m <= size + 1; --max: n = m = 1e5
if sys.argv[1] == "--max": print(10**5, 10**5); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(1, S + 1), r.randint(1, S + 1))
