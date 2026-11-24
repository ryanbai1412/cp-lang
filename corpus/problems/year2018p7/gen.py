"""year2018p7 generator: N <= 20000 vertices, parent of vertex i + 1 below i + 1, then N DNA
words over GCAT with total length S <= 1e5."""
import random
import sys


def emit(rng, n, total, alphabet, shape):
    parents = [rng.randrange(max(1, i - 1), i + 1) if shape == "long" else rng.randint(1, i) for i in range(1, n)]
    lens = [1] * n
    for _ in range(total - n):
        lens[rng.randrange(n)] += 1
    print(n)
    if parents:
        print("\n".join(map(str, parents)))
    print("\n".join("".join(rng.choice(alphabet) for _ in range(k)) for k in lens))


if sys.argv[1] == "--max":
    rng = random.Random(0)
    n = 20000
    # A broom: two long words sharing a 30000-letter block, at the ends of two branches.
    print(n)
    print("\n".join(["1"] + [str(i) for i in range(2, n - 1)] + ["1"]))
    block = "".join(rng.choice("GCAT") for _ in range(30000))
    rest = 100000 - 2 * 35000
    words = ["".join(rng.choice("GCAT") for _ in range(rest // (n - 2) + (i < rest % (n - 2)))) for i in range(n - 2)]
    print(words[0])
    print("\n".join(words[1:] + ["GA" * 1000 + block + "TC" * 1500, "CT" * 1500 + block + "AG" * 1000]))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(1, max(1, min(20000, size)))
emit(rng, n, rng.randint(n, max(n, min(100000, 4 * size))), rng.choice(["A", "AC", "GCAT"]), rng.choice(["long", "random"]))
