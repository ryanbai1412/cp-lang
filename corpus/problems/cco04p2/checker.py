"""cco04p2 checker: `checker.py input output answer`. Per test, the answer's game count m, then m
games: each a line of `a-b` scores that can be ordered with both numbers non-decreasing, every
score one Billy recorded (in either order), and every recorded score in some game."""
import sys

inp = open(sys.argv[1]).read().split()
out = [l.split() for l in open(sys.argv[2]).read().splitlines() if l.strip()]
ans = [l.split() for l in open(sys.argv[3]).read().splitlines() if l.strip()]


def score(t):
    x, _, y = t.partition("-")
    if not (x.isdigit() and y.isdigit()):
        sys.exit(1)
    return int(x), int(y)


pos, row, arow = 1, 0, 0
for _ in range(int(inp[0])):
    s = int(inp[pos])
    seen = {tuple(sorted(score(t))) for t in inp[pos + 1: pos + 1 + s]}
    pos += 1 + s
    m = int(ans[arow][0])
    arow += 1 + m
    if row >= len(out) or out[row] != [str(m)]:
        sys.exit(1)
    games, row = out[row + 1: row + 1 + m], row + 1 + m
    if len(games) != m:
        sys.exit(1)
    used = set()
    for g in games:
        game = sorted(score(t) for t in g)
        if any(b2 < b1 for (_, b1), (_, b2) in zip(game, game[1:])):
            sys.exit(1)
        used |= {tuple(sorted(p)) for p in game}
    if used != seen:
        sys.exit(1)
sys.exit(0 if row == len(out) else 1)
