"""sac22cc2p3 generator: N <= 17, R <= 1e9, then the 2^N - 1 forks `u v w D` of a perfect
binary decision tree rooted at node 1 over labels 1..=2^(N+1) - 1, |D| <= 10000."""
import random
import sys


def emit(rng, n, r, d):
    labels = list(range(2, 2 ** (n + 1)))
    rng.shuffle(labels)
    label = [0, 1] + labels  # heap position -> node label
    lines = [(label[i], label[2 * i], label[2 * i + 1], rng.randint(-d, d)) for i in range(1, 2 ** n)]
    rng.shuffle(lines)
    print(n, r)
    print("\n".join(" ".join(map(str, l)) for l in lines))


if sys.argv[1] == "--max":
    emit(random.Random(0), 17, 10**9, 10000)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
emit(rng, rng.randint(1, max(1, min(17, size.bit_length()))), rng.randint(0, 10**9), rng.choice([10, 10000]))
