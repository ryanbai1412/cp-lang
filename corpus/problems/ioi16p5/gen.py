import random
import sys

# `seed size`: n = 2^b <= max(2, size) (`--max`: n = 128), w = r = n b, then the hidden
# permutation p (an element x compiles to y with y[i] = x[p[i]]).
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "128"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
b = 7 if size >= 128 else r.randint(1, max(1, min(7, size.bit_length() - 1)))
n = 1 << b
p = list(range(n))
r.shuffle(p)
print(n, n * b, n * b)
print(*p)
