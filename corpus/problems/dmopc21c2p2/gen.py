"""gen.py seed size: DMOPC '21 C2 P2. 1 <= n <= 300, 1 <= q <= 3e5, at most 3000 Q queries.
size >= 3e5 gives n = 300, q = 3e5 with 3000 Q queries spread through the swaps."""
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 300, q = 3e5 with the full 3000 Q queries, each walking the whole permutation
    sys.argv[1:] = ['0', '300000']
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 3 * 10 ** 5
n = 300 if big else rng.randint(1, max(1, min(size, 6)))
q = 3 * 10 ** 5 if big else rng.randint(1, max(1, min(size, 12)))
asks = set(rng.sample(range(q), min(q, 3000))) if big else {i for i in range(q) if rng.random() < 0.3}
out = [f"{n} {q}"]
for i in range(q):
    if i in asks:
        p = list(range(1, n + 1)); rng.shuffle(p)
        out.append("Q " + " ".join(map(str, p)))
    else:
        out.append(f"{rng.choice('BE')} {rng.randint(1, n)} {rng.randint(1, n)}")
print("\n".join(out))
