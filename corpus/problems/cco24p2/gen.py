import sys, random
if sys.argv[1:] == ["--max"]:
    # N = 1e6 distinct flavours (the judged subtask at full size), people in random order: many
    # stacks, each placement a binary search over the stack fronts
    g = random.Random(1)
    n = 10**6
    a = list(range(1, n + 1)); g.shuffle(a)
    b = a[:]; g.shuffle(b)
    print(n); print(*a); print(*b)
    sys.exit()

# seed size: N <= size (capped at 1e6), inputs from the judged subtasks only: flavours <= 2, or distinct a
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10**6
n = 10**6 if big else r.randint(1, S)
kind = r.randint(0, 1)
if kind == 0:
    a = [r.randint(1, min(2, n)) for _ in range(n)] if r.random() < .9 else [1] * n
    mode = r.randint(0, 3)
    if mode == 0: b = a[::-1]
    elif mode == 1: b = a[:]
    else: b = a[:]; r.shuffle(b)
    if mode == 3 and not big: b[r.randrange(n)] = r.randint(1, min(2, n))
else:
    a = r.sample(range(1, n + 1), n)
    mode = r.randint(0, 4)
    if mode == 0: b = a[::-1]
    elif mode == 1: b = a[:]
    elif mode == 2:  # few long decreasing chains
        b = a[::-1]
        for _ in range(r.randint(1, 5)):
            i, j = r.randrange(n), r.randrange(n); b[i], b[j] = b[j], b[i]
    else: b = a[:]; r.shuffle(b)
    if mode == 4 and not big and r.random() < .5: b[r.randrange(n)] = r.randint(1, n)
print(n); print(*a); print(*b)
