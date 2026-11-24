// expect: E0701
// rule: 08.1.4
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/a.cp "pub fn tot(self: Range<Int>) -> Int = 1\n"
// file: kit/b.cp "pub fn tot(self: Range<Int>) -> Int = 2\n"
// Two imported fns that tie stay ambiguous: neither is the calling module's.
use kit.a.{tot}
use kit.b.{tot}
print((0..3).tot()) // <- here
