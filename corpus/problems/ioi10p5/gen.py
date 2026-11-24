"""ioi10p5 (Memory) hidden deals for interactor.py: `gen.py SEED SIZE`, or `gen.py --max`.

One line of 50 letters, the faces of cards 1..50: each of A..Y exactly twice, shuffled.
SIZE does not scale a game (it is always 50 cards); small sizes keep pairs closer together.
--max: the pairs as far apart as can be (A..Y then A..Y).
"""
import random
import string
import sys

MAX = sys.argv[1] == "--max"
letters = string.ascii_uppercase[:25]
if MAX:
    cards = letters + letters
else:
    rng = random.Random(int(sys.argv[1]))
    cards = list(letters * 2)
    rng.shuffle(cards)
    if int(sys.argv[2]) < 8:
        cards = sorted(cards, key=lambda c: (rng.random() < 0.5, c))
    cards = "".join(cards)
print(cards)
