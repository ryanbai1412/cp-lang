"""wac6p6 hidden data: T = 100 factories, each "A B C" (A + B + C = 100, in no colour order)
and its 100 canes in arrival order."""
import random
import sys


def factory(rng, counts, order):
    cols = rng.sample("RGB", 3)
    canes = [c for c, k in zip(cols, counts) for _ in range(k)]
    if order == "shuffle":
        rng.shuffle(canes)
    elif order == "reverse":
        canes.reverse()
    elif order == "interleave":
        seen = {}
        ranked = []
        for c in canes:  # colours take turns: each colour's k-th cane together
            seen[c] = seen.get(c, 0) + 1
            ranked.append((seen[c], cols.index(c), c))
        canes = [c for *_, c in sorted(ranked)]
    shown = list(counts)
    rng.shuffle(shown)
    return f"{' '.join(map(str, shown))}\n{''.join(canes)}"


def split(rng, skew):
    a, b = sorted(rng.randint(0, 100) for _ in range(2))
    counts = [a, b - a, 100 - b]
    if skew:  # the smallest two near the thresholds of 46
        a = rng.randint(0, 46)
        b = rng.randint(max(0, 36 - a), min(100 - a, 56 - a))
        counts = [a, b, 100 - a - b]
    return counts


if sys.argv[1] == "--max":
    rng = random.Random(0)
    print(100)
    print("\n".join(factory(rng, [a, b, 100 - a - b], ("shuffle", "reverse", "interleave")[a % 3])
                    for a, b in ((i % 34, 23 + i % 24) for i in range(100))))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
skew = int(sys.argv[2]) > 8
print(100)
print("\n".join(factory(rng, split(rng, skew and rng.random() < 0.7), rng.choice(["shuffle", "shuffle", "reverse", "interleave"]))
                for _ in range(100)))
