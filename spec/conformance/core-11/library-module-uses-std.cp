// expect: ok, output "2"
// rule: 11.3.1
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "use std.math.{gcd}\npub fn g() -> Int = gcd(4, 6)\n"
// A library module names what it uses from std.
use kit.m.{g}
print(g())
