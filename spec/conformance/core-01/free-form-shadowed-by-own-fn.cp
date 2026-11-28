// expect: ok, output "7 40"
// rule: 01.10.1
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/t.cp "pub trait Score:\n    fn base(self) -> Int\n    @free_form\n    fn score(self, bonus: Int = 0) -> Int = self.base() * 10 + bonus\n"
// The free name comes in with its trait as a glob import would bring it, so the module's own fn of that name shadows it; the method spelling still calls the trait's.
use kit.t.{Score}
struct P(n: Int)
impl Score for P:
    fn base(self) -> Int = self.n
fn score(p: P) -> Int = 7
print(score(P(4)), P(4).score())
