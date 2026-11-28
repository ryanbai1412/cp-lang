// expect: ok, output "40 40 42"
// rule: 01.10.1
// engines: next
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/t.cp "pub trait Score:\n    fn base(self) -> Int\n    @free_form\n    fn score(self, bonus: Int = 0) -> Int = self.base() * 10 + bonus\n"
// A library trait's `@free_form` method is called as a method and as a plain fn of its receiver alike.
use kit.t.{Score}
struct P(n: Int)
impl Score for P:
    fn base(self) -> Int = self.n
print(P(4).score(), score(P(4)), score(P(4), bonus: 2))
