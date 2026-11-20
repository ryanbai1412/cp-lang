"""Judge for yac6p3: two distinct hidden cells of an n by m grid; "? x y" -> Manhattan distance to the
nearer one (at most 5 questions, what the accepted original needs); "! x1 y1 x2 y2" must name both."""
class Judge:
    def __init__(self, input_text, rng):
        self.n, self.m = (int(x) for x in input_text.split()[:2])
        cells = set()
        while len(cells) < 2:
            cells.add((rng.randint(1, self.n), rng.randint(1, self.m)))
        self.cells, self.asked, self.verdict = sorted(cells), 0, False
    def first(self):
        return f"{self.n} {self.m}\n"
    def reply(self, line):
        t = line.split()
        if t[0] == "?":
            self.asked += 1
            x, y = int(t[1]), int(t[2])
            if self.asked > 5 or not (1 <= x <= self.n and 1 <= y <= self.m):
                return "-1"
            return str(min(abs(x - a) + abs(y - b) for a, b in self.cells))
        got = [int(v) for v in t[1:]]
        self.verdict = t[0] == "!" and len(got) == 4 and sorted([(got[0], got[1]), (got[2], got[3])]) == self.cells
        return None
    def ok(self):
        return self.verdict
