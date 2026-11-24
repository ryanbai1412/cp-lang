// expect: E0201
// rule: 11.3.1
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub fn g() -> Int = gcd(4, 6)\n"
// has: "gcd"
// A library module does not see the prelude: `gcd` there needs a `use`.
use kit.m.{g}
print(g()) // <- here
