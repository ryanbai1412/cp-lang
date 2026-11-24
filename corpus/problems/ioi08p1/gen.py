"""ioi08p1 (Type Printer) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N (1 <= N <= 25000) distinct words of 1..=20 lowercase letters.
--max: 25000 distinct random words of length 20, so the trie holds about 480000 letters.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, letters = 25_000, "abcdefghijklmnopqrstuvwxyz"
else:
    size = max(1, int(sys.argv[2]))
    letters = "abcdefghijklmnopqrstuvwxyz"[:rng.choice([1, 2, 3, 26])]
    longest = rng.randint(1, 20)
    n = min(rng.randint(1, min(25_000, size)), sum(len(letters) ** k for k in range(1, longest + 1)))
words = set()
while len(words) < n:
    length = 20 if MAX else rng.randint(1, longest)
    words.add("".join(rng.choice(letters) for _ in range(length)))
words = sorted(words)
rng.shuffle(words)
print(len(words))
print("\n".join(words))
