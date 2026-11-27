// expect: E0701
// rule: R08.1.4
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/a.cp "pub fn tot(x: Int) -> Int = 1\n"
// file: kit/b.cp "pub fn tot(x: Int) -> Int = 2\n"
// Two `use` lines that import one name as two fns leave a plain call of it ambiguous.
use kit.a.{tot}
use kit.b.{tot}
print(tot(3)) // <- here
