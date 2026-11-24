"""noimps22p3 generator: T cases of `N M`, N distinct names, M messages `s1 s2 s3`; every name posts at
least one academic message; 1 <= T <= 100, 1 <= N <= M <= 77777, sum M <= 2.5e5, strings of
[A-Za-z_?!.] at most 12 long."""
import random
import sys

ALPHA = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_?!."


def names(rng, n):
    out, seen = [], set()
    while len(out) < n:
        s = "".join(rng.choice(ALPHA) for _ in range(rng.randint(1, 12)))
        if s not in seen and s not in ("louxia", "loushang", "Louxia", "Xue", "Shu"):
            seen.add(s), out.append(s)
    return out


def case(rng, n, m, mutual):
    who = names(rng, n)
    msgs = [(p, rng.choice(["Xue", "Shu", "louxia", "loushang", "Louxia"]), rng.choice(["Shu", "LOUSHANG", "louxia"])) for p in who]
    msgs = [(p, "Xue" if b in who else b, c) if c in ("louxia", "loushang") else (p, b, c) for p, b, c in msgs]
    hot = who[: max(1, min(n, rng.choice([1, 2, 3, n])))]
    while len(msgs) < m:
        a, b = rng.choice(who), rng.choice(hot if rng.random() < 0.5 else who)
        r = rng.random()
        if mutual and r < 0.25:
            msgs.append((a, b, "loushang"))
            if len(msgs) < m:
                msgs.append((b, a, "louxia"))
        elif r < 0.6:
            msgs.append((a, b, "louxia"))
        elif r < 0.9:
            msgs.append((a, b, "loushang"))
        else:
            msgs.append((a, rng.choice([b, "Xue", "louxia"]), rng.choice(["Loushang", "louxia", "loushang", "XueShu"])))
    rng.shuffle(msgs)
    lines = [f"{n} {m}", *who] + [" ".join(x) for x in msgs]
    return lines


def emit(lines_per_case):
    print(len(lines_per_case))
    for lines in lines_per_case:
        print("\n".join(lines))


if sys.argv[1] == "--max":
    rng = random.Random(0)
    # Sum M = 2.5e5 in cases at M = 77777: few names (dense hubs), many names, mutual pairs.
    emit([case(rng, 3, 77777, True), case(rng, 20000, 77777, False), case(rng, 300, 77777, True), case(rng, 5000, 16669, True)])
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2])
    assert size >= 1
    rng = random.Random(seed)
    cap = min(77777, max(1, size))
    t = rng.randint(1, min(100, max(1, 250000 // cap)))
    if size <= 16:
        t = rng.randint(1, 5)
    cases = []
    for _ in range(t):
        m = rng.randint(1, cap)
        cases.append(case(rng, rng.randint(1, max(1, min(m, rng.choice([1, 2, 3, m // 2 + 1, m])))), m, rng.random() < 0.6))
    emit(cases)
