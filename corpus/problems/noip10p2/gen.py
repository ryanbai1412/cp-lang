"""gen.py seed size: NOIP 2010 P2. 1 <= N <= 350, 1 <= M <= 120, 0 <= a_i <= 100,
cards 1..4, at most 40 of each, summing to N - 1. size >= 350 gives the largest
state spaces: up to 40 of each card with N - 1 >= 329 (at most 120 cards fit)."""
import random, sys
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "350"]
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
while True:
    if size >= 350:
        cnt = [rng.randint(0, 40) for _ in range(4)]
    else:
        cnt = [rng.randint(0, min(40, size)) for _ in range(4)]
    s = sum((k + 1) * c for k, c in enumerate(cnt))
    if 1 <= sum(cnt) <= 120 and s + 1 <= 350 and (size < 350 or s + 1 >= 330):
        break
n = s + 1
cards = [k + 1 for k, c in enumerate(cnt) for _ in range(c)]
rng.shuffle(cards)
hi = 100 if size >= 350 or rng.random() < 0.5 else 3
print(n, len(cards))
print(" ".join(str(100 if size >= 350 and rng.random() < 0.3 else rng.randint(0, hi)) for _ in range(n)))
print(" ".join(map(str, cards)))
