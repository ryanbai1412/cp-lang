#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2003 Stick Game. One line of at most 1000 characters ending in
'#': a false equation of numbers below 1e7 joined by + and - with one '='; its one-stick fix is
unique when it exists. --max: 999 characters of seven-digit numbers built from the digits that
can lose or gain a stick, with no fix (so every pair of digits is tried)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else max(3, min(999, int(sys.argv[2])))
STICKS = [6, 2, 5, 5, 4, 5, 6, 3, 7, 6]
RESHAPE = {(0, 6), (0, 9), (2, 3), (3, 5), (6, 9)}
ONE_OFF = {(0, 8), (1, 7), (3, 9), (5, 6), (5, 9), (6, 8), (8, 9)}
link = lambda p, a, b: (min(a, b), max(a, b)) in p

def fixes(eq):
    digits, total, side, i = [], 0, 1, 0
    while i < len(eq):
        sign = side
        if eq[i] == "=": side, sign, i = -1, -1, i + 1
        if eq[i] in "+-":
            if eq[i] == "-": sign = -sign
            i += 1
        s = i
        while i < len(eq) and eq[i].isdigit(): i += 1
        place = sign
        for at in range(i - 1, s - 1, -1):
            digits.append((place, int(eq[at]), at)); total += place * int(eq[at]); place *= 10
    count = sum(1 for w, v, _ in digits for d in range(10) if link(RESHAPE, v, d) and total + (d - v) * w == 0)
    gain = {}
    for w, v, at in digits:
        for y in range(10):
            if STICKS[y] == STICKS[v] + 1 and link(ONE_OFF, v, y):
                gain.setdefault((y - v) * w, []).append(at)
    for w, v, at in digits:
        for x in range(10):
            if STICKS[x] == STICKS[v] - 1 and link(ONE_OFF, v, x):
                count += sum(1 for b in gain.get(-total - (x - v) * w, []) if b != at)
    return count

def number(alphabet, width):
    return r.choice(alphabet.lstrip("0")) + "".join(r.choice(alphabet) for _ in range(width - 1))

while True:
    if MAX:
        nums = [number("0135689", 7) for _ in range(125)]
        line = "".join(("" if i == 0 else "=" if i == 62 else r.choice("+-")) + x for i, x in enumerate(nums))[:999]
        line = line.rstrip("+-=")
    else:
        k = r.randint(2, max(2, S // 4))
        nums = [str(r.randint(0, 10 ** r.randint(1, 3) - 1)) for _ in range(k)]
        eq = r.randrange(1, k)
        if r.random() < 0.5:
            # a true equation with one stick moved away from it
            left = nums[:eq]; val = int(left[0]) + sum(int(x) for x in left[1:])
            nums = left + [str(val)]; eq = len(left)
        line = "".join(("" if i == 0 else "=" if i == eq else "+") + x for i, x in enumerate(nums))
        if r.random() < 0.5:
            pos = [i for i, c in enumerate(line) if c.isdigit()]
            i = r.choice(pos); c = int(line[i])
            opts = [d for d in range(10) if link(RESHAPE, c, d)]
            if opts: line = line[:i] + str(r.choice(opts)) + line[i + 1:]
    if "=" in line and line.count("=") == 1 and line[-1].isdigit() and fixes(line) <= 1:
        if MAX and fixes(line) != 0: continue
        print(line + "#")
        break
