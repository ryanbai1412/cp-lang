"""Judge for cpc21c1p5: n hidden rectangles with sides on 1..10^9, repeats allowed. "? x1 x2 y1 y2" is
answered with how many hidden rectangles lie inside that box; after "!" the solution prints all n."""
class Judge:
    def __init__(self, input_text, rng):
        self.n = int(input_text.split()[0])
        top = rng.choice([4, 12, 10**9])
        def side():
            a, b = sorted(rng.randint(1, top) for _ in range(2))
            return a, b
        self.hidden = sorted(side() + side() for _ in range(self.n))
        self.said, self.done = [], False
    def first(self):
        return f"{self.n}\n"
    def reply(self, line):
        kind, *nums = line.split()
        if kind == "?":
            x1, x2, y1, y2 = (int(x) for x in nums)
            if not (1 <= x1 <= x2 <= 10**9 and 1 <= y1 <= y2 <= 10**9):
                return None
            return str(sum(x1 <= a and b <= x2 and y1 <= c and d <= y2 for a, b, c, d in self.hidden))
        if kind == "!":
            self.done = True
            return "" if self.n else None
        self.said.append(tuple(int(x) for x in [kind] + nums))
        return "" if len(self.said) < self.n else None
    def ok(self):
        return self.done and sorted(self.said) == self.hidden
