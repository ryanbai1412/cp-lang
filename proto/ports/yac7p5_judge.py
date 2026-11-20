"""Judge for yac7p5: egg e breaks from floor a[e] upward, a is a hidden permutation of 1..n;
"? e f" -> 1 if egg e breaks at floor f, else 0; "! a..." must be the permutation.
Checks the answer and a loose question budget only, not the contest's scoring."""
class Judge:
    def __init__(self, input_text, rng):
        self.n = int(input_text.split()[0])
        self.a = list(range(1, self.n + 1)); rng.shuffle(self.a)
        self.asked, self.verdict = 0, False
    def first(self):
        return f"{self.n}\n"
    def reply(self, line):
        t = line.split()
        if t[0] == "?":
            self.asked += 1
            e, f = int(t[1]), int(t[2])
            if self.asked > 20 * self.n + 20 or not (1 <= e <= self.n and 1 <= f <= self.n):
                return "-1"
            return "1" if self.a[e - 1] <= f else "0"
        self.verdict = t[0] == "!" and [int(x) for x in t[1:]] == self.a
        return None
    def ok(self):
        return self.verdict
