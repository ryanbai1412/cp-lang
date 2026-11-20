"""Judge for yac6p3: two hidden cells of an n x m grid. "? x y" is answered with the walking
distance from (x, y) to the nearer hidden cell; "! x1 y1 x2 y2" must name both. At most 5 questions."""
class Judge:
    def __init__(self, input_text, rng):
        self.n, self.m = (int(x) for x in input_text.split()[:2])
        cells = self.n * self.m
        picks = [rng.randrange(cells) for _ in range(2)]
        if cells > 1:
            picks = rng.sample(range(cells), 2)
        self.hidden = sorted((p // self.m + 1, p % self.m + 1) for p in picks)
        self.asked, self.good = 0, False
    def first(self):
        return f"{self.n} {self.m}\n"
    def reply(self, line):
        kind, *nums = line.split()
        nums = [int(x) for x in nums]
        if kind == "?":
            x, y = nums
            self.asked += 1
            if self.asked > 5 or not (1 <= x <= self.n and 1 <= y <= self.m):
                return None
            return str(min(abs(x - p) + abs(y - q) for p, q in self.hidden))
        self.good = sorted([(nums[0], nums[1]), (nums[2], nums[3])]) == self.hidden
        return None
    def ok(self):
        return self.good
