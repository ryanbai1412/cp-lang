"""noip19p2 generator: n <= 5e5, a bracket per vertex, then parents f_2..f_n with f_i < i."""
import random
import sys


def emit(rng, n, chain, bias):
    print(n)
    print("".join("(" if rng.random() < bias else ")" for _ in range(n)))
    print(*(i - 1 if chain else rng.randint(max(1, i - 3), i - 1) if rng.random() < 0.9 else rng.randint(1, i - 1)
            for i in range(2, n + 1)))


if sys.argv[1] == "--max":
    rng = random.Random(0)
    n = 500_000
    print(n)
    # A path of "()()..." makes every prefix a long run of balanced chunks.
    print("".join("()"[i % 2] if i < n - 1000 else rng.choice("()") for i in range(n)))
    print(*range(1, n))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
emit(rng, rng.randint(1, max(1, min(500_000, size))), rng.random() < 0.3, rng.choice([0.5, 0.6]))
