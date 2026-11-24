#!/usr/bin/env python3
# checker.py input output [answer]: the input is a subtask (1-5) and the output an odometer program,
# parsed by the statement's rules and run, as the IOI simulator does, on a 256 x 256 grid for each
# test grid below; it must stay within the subtask's program size and execution length and end in
# the required state. Subtask 4 and 5 limits are where the score reaches 0 (2e6 steps, size 4440).
import random
import re
import sys

N = 256
LIMITS = {1: (100, 1000), 2: (200, 2000), 3: (100, 200_000), 4: (200, 1_999_999), 5: (4439, 44_400_000)}
LABEL = re.compile(r"[A-Za-z0-9_]{1,128}")
DIRS = [(-1, 0), (0, 1), (1, 0), (0, -1)]  # north, east, south, west


def parse(text):
    ops, labels, uses = [], {}, []
    for line in text.split("\n"):
        line = line.split("#", 1)[0].split()
        if not line:
            continue
        if len(line) == 1 and line[0].endswith(":"):
            name = line[0][:-1]
            if not LABEL.fullmatch(name) or name in labels:
                return None
            labels[name] = len(ops)
        elif len(line) == 1 and line[0] in ("left", "right", "move", "get", "put", "halt"):
            ops.append([line[0], None])
        elif len(line) == 2 and line[0] in ("jump", "border", "pebble") and LABEL.fullmatch(line[1]):
            ops.append([line[0], line[1]])
            uses.append(line[1])
        else:
            return None
    if any(u not in labels for u in uses):
        return None
    code = {"left": 0, "right": 1, "move": 2, "get": 3, "put": 4, "halt": 5, "jump": 6, "border": 7, "pebble": 8}
    return [(code[o], labels[a] if a else 0) for o, a in ops]


def run(prog, cells, cap):
    """Returns (row, col, grid) at the end, or None past cap steps."""
    g = [0] * (N * N)
    for (r, c), p in cells.items():
        g[r * N + c] = p
    r = c = d = 0
    pc, steps, end = 0, 0, len(prog)
    while pc < end:
        steps += 1
        if steps > cap:
            return None
        op, t = prog[pc]
        pc += 1
        if op == 8:
            if g[r * N + c]:
                pc = t
        elif op == 7:
            dr, dc = DIRS[d]
            if not (0 <= r + dr < N and 0 <= c + dc < N):
                pc = t
        elif op == 2:
            dr, dc = DIRS[d]
            if 0 <= r + dr < N and 0 <= c + dc < N:
                r += dr
                c += dc
        elif op == 6:
            pc = t
        elif op == 3:
            if g[r * N + c]:
                g[r * N + c] -= 1
        elif op == 4:
            if g[r * N + c] < 15:
                g[r * N + c] += 1
        elif op == 0:
            d = (d + 3) % 4
        elif op == 1:
            d = (d + 1) % 4
        else:
            break
    return r, c, g


def grids(task, rng):
    if task in (1, 2):
        for x in range(16):
            for y in range(16):
                yield {(0, 0): x, (0, 1): y}
    elif task == 3:
        pairs = [(0, 2), (0, 254), (1, 255), (253, 255), (254, 0), (255, 1), (100, 102), (0, 128)]
        while len(pairs) < 40:
            x, y = rng.sample(range(N), 2)
            if (x + y) % 2 == 0:
                pairs.append((x, y))
        for x, y in pairs:
            yield {(0, x): 1, (0, y): 1}
    elif task == 4:
        yield {}
        yield {(0, 0): 1}
        yield {(N - 1, N - 1): 1}
        yield {(0, c): 1 for c in range(N - 15, N)}
        yield {(r, N - 1): 1 for r in range(N - 15, N)}
        yield {(N - 1 - i, N - 1 - j): 1 for i in range(4) for j in range(4) if i + j}
        for _ in range(4):
            k = rng.randint(1, 15)
            yield {cell: 1 for cell in rng.sample([(r, c) for r in range(N) for c in range(N)], k)}
    else:
        yield {(r, c): 15 for r in range(N) for c in range(N)}
        yield {}
        yield {(r, c): rng.randint(0, 15) for r in range(N) for c in range(N)}
        for low, at in ((3, (N - 1, 0)), (9, (N - 1, N - 1)), (14, (0, N - 1))):
            g = {(r, c): rng.randint(low + 1, 15) for r in range(N) for c in range(N)}
            g[at] = low
            g[(rng.randrange(N), rng.randrange(N))] = low
            yield g


def ok(task, cells, res):
    r, c, g = res
    if task == 1 or task == 2:
        x, y = cells[(0, 0)], cells[(0, 1)]
        if (r, c) != ((0, 0) if x <= y else (0, 1)):
            return False
        return task == 1 or (g[0], g[1]) == (x, y)
    if task == 3:
        xs = [col for (_, col) in cells]
        return (r, c) == (0, sum(xs) // 2)
    if task == 4:
        return g[0] == len(cells) and not any(g[1:])
    want = [0] * (N * N)
    for (rr, cc), p in cells.items():
        want[rr * N + cc] = p
    return g == want and g[r * N + c] == min(g)


task = int(open(sys.argv[1]).read().split()[0])
text = open(sys.argv[2], encoding="utf-8", errors="replace").read()
prog = parse(text) if len(text.encode()) <= 5 << 20 else None
size, cap = LIMITS[task]
if prog is None or len(prog) > size:
    sys.exit(1)
rng = random.Random(task)
for cells in grids(task, rng):
    res = run(prog, cells, cap)
    if res is None or not ok(task, cells, res):
        sys.exit(1)
sys.exit(0)
