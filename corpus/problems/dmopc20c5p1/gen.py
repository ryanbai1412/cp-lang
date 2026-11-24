"""gen.py SEED SIZE | --max: DMOPC '20 Contest 5 P1 (Home Row). Lowercase S and T, each of
length 1..=1e6, usually sharing a prefix (sometimes one is a prefix of the other). --max is
|S| = |T| = 1e6 that agree on all but the last letter."""
import random, sys
if sys.argv[1] == "--max":
    sys.stdout.write("a" * 10**6 + "\n" + "a" * (10**6 - 1) + "b\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
cap = min(10**6, size)
alpha = "abcdefghijklmnopqrstuvwxyz"[: r.choice([1, 2, 26])]
word = lambda k: "".join(r.choice(alpha) for _ in range(k))
p = word(r.randint(0, cap - 1))
s = p + word(r.randint(1 if not p else 0, cap - len(p)))
t = p + word(r.randint(1 if not p else 0, cap - len(p)))
if r.random() < 0.2:
    t = s[: r.randint(1, len(s))]
sys.stdout.write(f"{s}\n{t}\n")
