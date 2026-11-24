// expect: ok, output "7 2"
// rule: 08.1.4
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub fn tot(self: Range<Int>) -> Int = 1\npub fn two(self: Range<Int>) -> Int = 2\n"
// At equal specificity the calling module's own fn beats one imported from a library.
use kit.m.{two}
fn tot(self: Range<Int>) -> Int = 7
print((0..3).tot(), (0..3).two())
