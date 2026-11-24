import sys, random
# t tests; kind 1: n in 1..=67e6; kind 2: a card drawn from the 28 orbit segments, any order.
BASE = [((2, 0), (2, 1)), ((2, 1), (2, 2)), ((1, 1), (2, 1)), ((1, 1), (2, 0)),
        ((1, 2), (2, 1)), ((0, 0), (2, 1)), ((0, 0), (1, 2))]
turn = lambda d: (2 - d[1], d[0])
SEGS = []
for s in BASE:
    for _ in range(4):
        SEGS.append(s)
        s = (turn(s[0]), turn(s[1]))
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 10**5 if MAX else int(sys.argv[2])
t = 10**5 if MAX else r.randint(1, max(1, min(S, 30)))
out = [str(t)]
for _ in range(t):
    if r.random() < 0.5:
        n = r.choice([1, 2, 67_000_000, r.randint(1, 67_000_000), r.randint(1, 1000)])
        out += ["1", str(n)]
    else:
        card = [s for s in SEGS if r.random() < r.choice([0.1, 0.5, 0.9])]
        r.shuffle(card)
        out += ["2", str(len(card))]
        for a, b in card:
            if r.random() < 0.5: a, b = b, a
            out.append(f"{a[0]} {a[1]} {b[0]} {b[1]}")
print("\n".join(out))
