import sys, random
if sys.argv[1] == "--max":
    # one case n = 2e5 (sum n): skills rising by about 5000 up to 1e9, so every sorcerer is a top
    # (prefix maximum) and most meetings are forfeits; removals in random order.
    r = random.Random(0); n = 200000
    a = [5000 * (i + 1) - r.randint(0, 4999) for i in range(n)]
    p = list(range(1, n + 1)); r.shuffle(p)
    print(1); print(n); print(*a); print(*p); sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
tests = [size] if size > 100 else [rng.randint(1, 3 * size) for _ in range(rng.randint(1, 5))]
print(len(tests))
for n in tests:
    style = rng.randint(0, 2) if size <= 100 else int(sys.argv[1]) % 3
    if style == 0:
        a = [rng.randint(1, 10) for _ in range(n)]
    elif style == 1:
        a = [rng.randint(1, 2 ** rng.randint(0, 29)) for _ in range(n)]
    else:
        a, s = [], 1
        for _ in range(n):
            a.append(min(10 ** 9, rng.choice([s, s + 1, rng.randint(1, s)])))
            s = min(10 ** 9, s + a[-1])
            if rng.random() < 0.1: s = 1
    p = list(range(1, n + 1))
    rng.shuffle(p)
    print(n)
    print(*a)
    print(*p)
