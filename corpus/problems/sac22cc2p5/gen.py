"""gen.py seed size: SAC '22 CC2 P5. 1 <= N, Q <= 1e6, |A_i| <= 1e9; "1 i j" sets A_i = j
(|j| <= 1e9), "2 L R" with L <= R asks for A_L + A_{L+2} + ..."""
import random, sys

if "--max" in sys.argv[1:]:
    rng = random.Random(22)
    n = q = 10 ** 6
    cap = 10 ** 9
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2])
    rng = random.Random(seed)
    n, q = rng.randint(1, max(1, size)), rng.randint(1, max(1, size))
    cap = rng.choice([5, 10 ** 9])
out = [f"{n} {q}", " ".join(str(rng.randint(-cap, cap)) for _ in range(n))]
for _ in range(q):
    if rng.random() < 0.5:
        out.append(f"1 {rng.randint(1, n)} {rng.randint(-cap, cap)}")
    else:
        l = rng.randint(1, n)
        out.append(f"2 {l} {rng.randint(l, n)}")
sys.stdout.write("\n".join(out) + "\n")
