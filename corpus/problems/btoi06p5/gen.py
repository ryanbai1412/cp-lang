"""btoi06p5 (RLE Compression) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = 1e5, m = 2e6).

"n m" then a valid code of m symbols: plain symbols, "e e k" (k + 1 escapes), "e b 0"
(b becomes the escape) and "e b k" (k + 3 copies of b), with e the current escape.
--max: many escape switches and long runs of the escape and of other symbols.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 2 * 10**6 if MAX else max(1, int(sys.argv[2]))
n = 10**5 if MAX else rng.randint(2, max(2, min(size, rng.choice([2, 5, 100000]))))
m = size if MAX else rng.randint(1, size)
alpha = n if MAX else rng.randint(2, n)  # symbols actually used
code, esc = [], 0
while len(code) < m:
    left = m - len(code)
    pick = rng.random()
    if left < 3 or pick < 0.4:
        s = rng.randrange(alpha)
        if s != esc:
            code.append(s)
    elif pick < 0.6:
        code += [esc, esc, rng.randrange(n)]
    elif pick < 0.8:
        b = rng.randrange(alpha)
        if b != esc:
            code += [esc, b, 0]
            esc = b
    else:
        b = rng.randrange(alpha)
        if b != esc:
            code += [esc, b, rng.randint(1, n - 1)]
print(n, len(code))
print(" ".join(map(str, code)))
