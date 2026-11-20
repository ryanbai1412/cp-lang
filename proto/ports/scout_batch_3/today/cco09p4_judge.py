"""Judge for cco09p4: n caps and n bottles with hidden sizes. "0 c b" compares cap c with bottle b
(1 when the cap is bigger, 0 when it fits, -1 when it is smaller); "1 c b" claims a pair."""
class Judge:
    def __init__(self, input_text, rng):
        self.n = int(input_text.split()[0])
        self.cap = list(range(self.n)); rng.shuffle(self.cap)
        self.bottle = list(range(self.n)); rng.shuffle(self.bottle)
        self.claimed, self.bad = set(), False
    def first(self):
        return f"{self.n}\n"
    def reply(self, line):
        kind, c, b = (int(x) for x in line.split())
        if not (1 <= c <= self.n and 1 <= b <= self.n):
            self.bad = True
            return None
        if kind == 0:
            x, y = self.cap[c - 1], self.bottle[b - 1]
            return str((x > y) - (x < y))
        if self.cap[c - 1] != self.bottle[b - 1] or c in self.claimed:
            self.bad = True
            return None
        self.claimed.add(c)
        return None if len(self.claimed) == self.n else ""
    def ok(self):
        return not self.bad and len(self.claimed) == self.n
