// expect: E0701
// rule: 01.10.1
// engines: next
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/a.cp "pub trait A:\n    @free_form\n    fn score(self) -> Int = 1\n"
// file: kit/b.cp "pub trait B:\n    @free_form\n    fn score(self) -> Int = 2\n"
// has: "score"
// Two traits in scope that each give a free `score` make the plain call ambiguous; `x.score()` or `A.score(x)` says which.
use kit.a.{A}
use kit.b.{B}
struct P(n: Int)
impl A for P
print(score(P(4))) // <- here
