"""gen.py SEED SIZE | --max: NOIP 2021 P4. T in 1..=5 boards, 2 <= n, m <= 1e5 with
4 <= nm <= 2e5, edge states 0..=3, q <= min(1e5, nm) pieces on distinct cells with
1 <= lv_i <= q. Small sizes vary the edge mix (some boards use only some states, as the
statement's subtasks do) and the level order; --max is T = 5 boards of 447 x 447 (nm = 199809)
of mostly interchange and no-turn edges, each with q = 1e5 pieces of random levels."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
T = 5 if MAX else r.randint(1, 5)
out = [str(T)]
for _ in range(T):
    if MAX:
        n = m = 447
    else:
        cells = r.randint(4, min(200_000, 4 + 4 * S))
        n = r.randint(2, max(2, cells // 2))
        m = max(2, cells // n)
    q = 100_000 if MAX else r.randint(1, min(100_000, n * m))
    states = "0123" + "3" * 3 + "2" * 2 if MAX else r.choice(["0123", "01", "012", "013", "0123333", "0122223", "123"])
    out.append(f"{n} {m} {q}")
    out += ["".join(r.choice(states) for _ in range(m - 1)) for _ in range(n)]
    out += ["".join(r.choice(states) for _ in range(m)) for _ in range(n - 1)]
    cells = r.sample(range(n * m), q)
    order = r.choice([0, 0, 1, 2]) if not MAX else 0
    for i, cell in enumerate(cells):
        lv = r.randint(1, q) if order == 0 else i + 1 if order == 1 else q - i
        out.append(f"{r.randint(0, 1)} {lv} {cell // m + 1} {cell % m + 1}")
sys.stdout.write("\n".join(out) + "\n")
