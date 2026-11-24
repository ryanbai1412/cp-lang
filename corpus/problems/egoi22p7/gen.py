import sys, random
# n <= 200 pins, m hidden wires between distinct pins.
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 200 if MAX else int(sys.argv[2])
n = 200 if MAX else r.randint(1, max(1, min(S, 200)))
wires = []
if n > 1:
    style = 0 if MAX else r.randrange(4)
    if style == 0:  # many small components
        parts = r.randint(1, n)
        comp = [r.randrange(parts) for _ in range(n)]
        for _ in range(r.randint(0, 3 * n)):
            a, b = r.sample(range(n), 2)
            if comp[a] == comp[b]: wires.append((a + 1, b + 1))
    elif style == 1:  # a few big ones
        for _ in range(r.randint(0, n)):
            a, b = r.sample(range(n), 2); wires.append((a + 1, b + 1))
    elif style == 2:  # a path
        order = list(range(1, n + 1)); r.shuffle(order)
        wires = [(order[i], order[i + 1]) for i in range(n - 1)]
print(n, len(wires))
for a, b in wires: print(a, b)
