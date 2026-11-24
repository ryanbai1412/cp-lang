import sys, random
# statement: N <= 1000 hidden numbers 0 <= F_i < 2^60.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 1000
n = 1000 if big else r.randint(1, max(1, min(1000, S)))
def number():
    if big: return r.randrange(1 << 59, 1 << 60)
    return r.randrange(0, 1 << r.randint(0, 60))
print(n)
print(*(number() for _ in range(n)))
