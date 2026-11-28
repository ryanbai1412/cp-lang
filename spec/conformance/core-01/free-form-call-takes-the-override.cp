// expect: ok, output "99 10 99"
// rule: 01.10.1
// engines: next
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/t.cp "pub trait Score:\n    fn base(self) -> Int\n    @free_form\n    fn score(self, bonus: Int = 0) -> Int = self.base() * 10 + bonus\n"
// The plain call is method lookup on its first argument, so a type's override of the default is the call, through a bound too.
use kit.t.{Score}
struct P(n: Int)
impl Score for P:
    fn base(self) -> Int = self.n
struct Q(n: Int)
impl Score for Q:
    fn base(self) -> Int = self.n
    fn score(self, bonus: Int = 0) -> Int = 99
fn via<S: Score>(s: S) -> Int = score(s)
print(score(Q(0)), score(P(1)), via(Q(0)))
