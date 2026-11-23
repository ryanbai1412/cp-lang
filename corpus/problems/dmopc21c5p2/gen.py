"""dmopc21c5p2 generator: one n in 1..=1_000_000, small sizes favouring the edge cases 1..=4."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
print(rng.choice([1, 2, 3, 4, rng.randint(1, min(1_000_000, max(1, size)))]))
