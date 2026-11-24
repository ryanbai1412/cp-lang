import sys, random
# seed size: n <= min(64, size + 1) bits and any k < 2^n; --max: n = 64, k = 2^64 - 1
if sys.argv[1] == "--max": print(64, 2 ** 64 - 1); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(64, S + 1)); print(n, r.randrange(2 ** n))
