"""Judge for cco18p4: k hidden tokens on a rows x cols grid. "? r c" is answered with the total
walking distance from (r, c) to the tokens; "! v" must be the least such total. At most 170 questions."""
class Judge:
    def __init__(self, input_text, rng):
        self.rows, self.cols, self.k = (int(x) for x in input_text.split()[:3])
        self.tokens = [(rng.randint(1, self.rows), rng.randint(1, self.cols)) for _ in range(self.k)]
        self.asked, self.good = 0, False
    def cost(self, r, c):
        return sum(abs(r - x) + abs(c - y) for x, y in self.tokens)
    def first(self):
        return f"{self.rows} {self.cols} {self.k}\n"
    def reply(self, line):
        kind, *nums = line.split()
        if kind == "?":
            r, c = (int(x) for x in nums)
            self.asked += 1
            if self.asked > 170 or not (1 <= r <= self.rows and 1 <= c <= self.cols):
                return None
            return str(self.cost(r, c))
        xs = sorted(x for x, _ in self.tokens)
        ys = sorted(y for _, y in self.tokens)
        self.good = int(nums[0]) == self.cost(xs[len(xs) // 2], ys[len(ys) // 2])
        return None
    def ok(self):
        return self.good
