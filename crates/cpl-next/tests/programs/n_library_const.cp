// expect: ok, output "1000000007 3\n8 2"
// rule: R01.6.1 R11.2.1
// A library's typed `const` is read by its own fns and by a program that imports it.
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub const M: Int = 1_000_000_007\nconst STEP: Int = 3\npub struct Unit(v: Int)\npub const one: Unit = Unit(2)\npub fn wrap(x: Int) -> Int = (x + STEP) % M\npub fn step() -> Int = STEP\n"
use kit.m.{M, one, wrap, step}
print(M, step())
print(wrap(M + 5), one.v)
