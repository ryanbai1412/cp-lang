"""sac22cc4jp1 (Obligatory DeMello Problem) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

One 5-letter lowercase word W; good when it has 2 or more of aeiouy. The size is ignored:
every input is 5 letters.
--max: `yyyya`, every letter a vowel.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
vowels = rng.randint(0, 3)
w = [rng.choice("aeiouy") if i < vowels else rng.choice("bcdfghjklmnpqrstvwxz") for i in range(5)]
rng.shuffle(w)
print("yyyya" if MAX else "".join(w))
