import random
import sys

# `seed size`: n distinct binary words of 1..m bits, n, m <= 2 + 4 size, drawn from a few
# short pieces so that prefixes (and so ambiguous strings) are common.
# `--max`: n = m = 50 words built the same way.
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 50 if MAX else int(sys.argv[2])
n = 50 if MAX else r.randint(1, min(50, 2 + 4 * size))
m = 50 if MAX else r.randint(1, min(50, 2 + 4 * size))
pieces = ["".join(r.choice("01") for _ in range(r.randint(1, 3))) for _ in range(3)]
words = set()
for _ in range(1000):
    if len(words) == n:
        break
    w = ""
    while len(w) < r.randint(1, m):
        w += r.choice(pieces) if r.random() < 0.7 else r.choice("01")
    words.add(w[:m])
words = sorted(words)
r.shuffle(words)
print(len(words), m)
for w in words:
    print(w)
