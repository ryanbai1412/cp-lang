"""gen.py seed size: NOIP 2010 P2. 1 <= N <= 350, 1 <= M <= 120, 0 <= a_i <= 100,
cards 1..4, at most 40 of each, summing to N - 1. size >= 350 gives the largest
state spaces: up to 40 of each card with N - 1 >= 329 (at most 120 cards fit)."""
import random, sys
if sys.argv[1] == "--max":
    # N = 350 and M = 120 together: 120 cards of at most 40 per kind moving 349 squares, the
    # counts chosen for the most states (each count + 1, multiplied).
    best = max(((a, b, c, d) for a in range(41) for b in range(41) for c in range(41)
                for d in [120 - a - b - c] if 0 <= d <= 40 and a + 2 * b + 3 * c + 4 * d == 349),
               key=lambda t: (t[0] + 1) * (t[1] + 1) * (t[2] + 1) * (t[3] + 1))
    rng = random.Random(0)
    cards = [k + 1 for k, c in enumerate(best) for _ in range(c)]
    rng.shuffle(cards)
    print(350, len(cards))
    print(" ".join(str(100 if rng.random() < 0.3 else rng.randint(0, 100)) for _ in range(350)))
    print(" ".join(map(str, cards)))
    sys.exit()
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
