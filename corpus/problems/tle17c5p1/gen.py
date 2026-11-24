import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "50"]
# `seed size`: the secret N in -50..=50 (seeds 0 and 1 pin the ends).
seed = int(sys.argv[1])
print({0: -50, 1: 50}.get(seed, random.Random(seed).randint(-50, 50)))
