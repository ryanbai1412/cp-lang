"""gen.py SEED SIZE | --max: NOI 2005 Wisdom Beads. A 10-row triangle, line i of i cells, each a
letter A-L already placed or '.', with at most one way to finish it. A seed takes one of nine
full tilings (maybe mirrored), lifts out up to SIZE + 1 pieces while the completion stays
unique, and sometimes puts one lifted piece back somewhere else so no completion exists. --max places a lone
G whose completion is unique, so the other eleven pieces are searched from scratch."""
import random, sys

TILINGS = """BBKBKKBJKKJJJDDGJGDDCGGGCCCIEEEHHIIAELHHHIAAFELLLLIFFFF
BBKBKKBAKKCAADDCCCDDILLLLIIFGGJLIEFFGJJJIEFHHGGJEEEFHHH
HHHHHCLLCBLCCBKLDDBKKLDDBFKKGGJFFFFEGJJJIIIEAGGJIIEEEAA
LLALAALLIBCCIBKCIIBKKCIHBJKKEHHJJJDDEHHGJGDDFEEEGGGFFFF
BBABAABEEEFECCCFECGGGFFJGIGLFJJJIILKHHJDDILKKHHHDDILLKK
LLKLKKLLKKAACCCEACIIIEBBBBIIEEEJGGDDHHJJJGDDFHHHJGGFFFF
KKKCKKCIIICCHIIEHHGGJEHHGJJJEEEGGJAADDLBBBBAFDDLLLLFFFF
FFFFHHFHHAIHJAAIJJJCCIIJDDCEGIGDDCEKGGGLEEEKKLLLLBBBBKK
HHHHHFIFFKIIFKKBIFJKKBIJJJGGBDDJEGCCBDDLEGGCALLLLEEECAA""".split()
CELLS = [(i, j) for i in range(10) for j in range(i + 1)]


def rows(flat):
    return [flat[i * (i + 1) // 2:(i + 1) * (i + 2) // 2] for i in range(10)]


def orients(cells):
    out = set()
    for f in range(2):
        cs = [(x, -y) if f else (x, y) for x, y in cells]
        for _ in range(4):
            cs = [(y, -x) for x, y in cs]
            mx, my = min(x for x, _ in cs), min(y for _, y in cs)
            out.add(tuple(sorted((x - mx, y - my) for x, y in cs)))
    return out


FIG = rows(TILINGS[0])
SHAPES = {c: orients([p for p in CELLS if FIG[p[0]][p[1]] == c]) for c in "ABCDEFGHIJKL"}


def completions(board, limit=2):
    free = frozenset(p for p in CELLS if board[p[0]][p[1]] == ".")
    left = [c for c in "ABCDEFGHIJKL" if not any(c in r for r in board)]
    if sum(len(next(iter(SHAPES[c]))) for c in left) != len(free):
        return 0
    found = 0

    def rec(free, left):
        nonlocal found
        if not left:
            found += 1
            return
        cell = min(free)
        for c in left:
            for o in SHAPES[c]:
                ax, ay = o[0]
                pl = [(cell[0] + x - ax, cell[1] + y - ay) for x, y in o]
                if all(p in free for p in pl):
                    rec(free - set(pl), [d for d in left if d != c])
                    if found >= limit:
                        return

    rec(free, left)
    return found


if sys.argv[1] == "--max":
    print("\n".join(["." * (i + 1) for i in range(7)] + [".....GGG", ".....G.G.", ".........."]))
    sys.exit()
r = random.Random(int(sys.argv[1]))
g = [list(x) for x in rows(r.choice(TILINGS))]
if r.random() < 0.5:
    g = [[g[9 - j][9 - i] for j in range(i + 1)] for i in range(10)]
lift = min(7, 1 + int(sys.argv[2]))
order = list("ABCDEFGHIJKL")
r.shuffle(order)
for c in order[:lift]:
    trial = [["." if x == c else x for x in row] for row in g]
    if completions(trial) == 1:
        g = trial
gone = [c for c in order[:lift] if not any(c in row for row in g)]
if gone and r.random() < 0.25:
    c = r.choice(gone)
    free = [p for p in CELLS if g[p[0]][p[1]] == "."]
    spots = [[(i + x, j + y) for x, y in o] for o in sorted(SHAPES[c]) for i, j in free]
    for pl in r.sample(spots, len(spots)):
        if all(p in CELLS and g[p[0]][p[1]] == "." for p in pl):
            trial = [row[:] for row in g]
            for x, y in pl:
                trial[x][y] = c
            if completions(trial, 1) == 0:
                g = trial
                break
print("\n".join("".join(row) for row in g))
