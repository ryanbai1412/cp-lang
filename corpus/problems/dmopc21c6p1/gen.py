"""gen.py SEED SIZE | --max: DMOPC '21 Contest 6 P1 (Bigger Integers). D in 2..=1e6 and a
D-digit X of non-zero digits, often non-increasing with one rise so the swap is late.
--max is D = 1e6, non-increasing except a rise at the very end."""
import random, sys
if sys.argv[1] == "--max":
    d = 10**6
    sys.stdout.write(f"{d}\n" + "9" * (d - 2) + "12\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
d = r.randint(2, max(2, min(10**6, int(sys.argv[2]) + 1)))
digits = "123456789"[: r.choice([2, 3, 9])]
x = [r.choice(digits) for _ in range(d)]
if r.random() < 0.5:
    x.sort(reverse=True)
    if r.random() < 0.7:
        i = r.randrange(d)
        x[i] = r.choice(digits)
sys.stdout.write(f"{d}\n{''.join(x)}\n")
