"""Judge for aac5p2: a hidden permutation; "? i j" -> a[i] * a[j]; "! a..." must be a permutation
that fits every answer given (n = 2 is ambiguous, so equality with the hidden one is too strict)."""
class Judge:
    def __init__(self, input_text, rng):
        self.n = int(input_text.split()[0])
        self.a = list(range(1, self.n + 1)); rng.shuffle(self.a)
        self.asked, self.verdict, self.seen = 0, False, []
    def first(self):
        return f"{self.n}\n"
    def reply(self, line):
        t = line.split()
        if t[0] == "?":
            self.asked += 1
            i, j = int(t[1]), int(t[2])
            if self.asked > self.n or i == j or not (1 <= i <= self.n and 1 <= j <= self.n):
                return "-1"
            self.seen.append((i, j))
            return str(self.a[i - 1] * self.a[j - 1])
        b = [int(x) for x in t[1:]]
        self.verdict = (t[0] == "!" and sorted(b) == sorted(self.a)
                        and all(b[i - 1] * b[j - 1] == self.a[i - 1] * self.a[j - 1] for i, j in self.seen))
        return None
    def ok(self):
        return self.verdict
