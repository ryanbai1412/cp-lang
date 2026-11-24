import sys, random
# statement: 2 <= N <= 1e6.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(10 ** 6 if S >= 10 ** 6 else r.randint(2, max(2, min(10 ** 6, S))))
