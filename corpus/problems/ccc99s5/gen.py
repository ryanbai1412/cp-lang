"""gen.py SEED SIZE | --max: CCC 1999 S5. Puzzles word1 + word2 = word3 in capital letters, with
distinct letters given distinct digits and no leading zero, each with exactly one answer: a
random assignment is kept only when a column-by-column count finds no second answer. --max is
ten puzzles of 20-letter words over all ten digits."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])

def count(rows, cap=2):
    width = max(map(len, rows)); leads = {w[0] for w in rows}
    digit = {}; used = [False] * 10; found = 0
    def at(row, col):
        w = rows[row]; return w[len(w) - 1 - col] if col < len(w) else None
    def give(x, d, rest):
        if x in digit: return digit[x] == d and rest()
        if used[d] or (d == 0 and x in leads): return False
        digit[x] = d; used[d] = True
        ok = rest()
        del digit[x]; used[d] = False
        return ok
    def fill(col, row, total):
        nonlocal found
        if col == width:
            if total == 0: found += 1
            return found >= cap
        if row == 2:
            x = at(2, col)
            if x is None: return total % 10 == 0 and fill(col + 1, 0, total // 10)
            return give(x, total % 10, lambda: fill(col + 1, 0, total // 10))
        x = at(row, col)
        if x is None: return fill(col, row + 1, total)
        if x in digit: return fill(col, row + 1, total + digit[x])
        return any(give(x, d, lambda d=d: fill(col, row + 1, total + d)) for d in range(10))
    fill(0, 0, 0)
    return found

def puzzle():
    while True:
        k = 10 if MAX else r.randint(2, min(10, 3 + S))
        letters = r.sample("ABCDEFGHIJKLMNOPQRSTUVWXYZ", k)
        name = dict(zip(r.sample(range(10), k), letters))
        digits = sorted(name)
        top = 19 if MAX else r.randint(1, min(19, 2 + S))
        def num(length):
            s = [r.choice(digits) for _ in range(length)]
            if s[0] == 0: return None
            return int("".join(map(str, s)))
        a = num(top if MAX else r.randint(1, top)); b = num(top if MAX else r.randint(1, top))
        if a is None or b is None: continue
        c = str(a + b)
        if any(int(ch) not in name for ch in c): continue
        rows = ["".join(name[int(ch)] for ch in str(v)) for v in (a, b)] + ["".join(name[int(ch)] for ch in c)]
        if max(map(len, rows)) <= 20 and count(rows) == 1: return rows

t = 10 if MAX else r.randint(1, 3)
print(t)
for _ in range(t): print(*puzzle(), sep="\n")
