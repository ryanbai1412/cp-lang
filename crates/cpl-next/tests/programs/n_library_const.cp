// expect: ok, output "built\n1000000007 3\n8 2 4\n4"
// rule: R01.6.1 R11.2.1
// A library's `const` is set once, at program start, after the consts its initializer reads:
// `one` is built once however often it is read, and `AFTER` reads `BEFORE`, declared below it.
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub const M: Int = 1_000_000_007\nconst STEP: Int = 3\npub struct Unit(v: Int)\nfn make() -> Unit:\n    print(\"built\")\n    Unit(2)\npub const one: Unit = make()\npub const AFTER = BEFORE + 1\nconst BEFORE = STEP\npub fn wrap(x: Int) -> Int = (x + STEP) % M\npub fn step() -> Int = STEP\npub fn twice() -> Int = one.v + one.v\n"
use kit.m.{M, one, wrap, step, twice, AFTER}
print(M, step())
print(wrap(M + 5), one.v, twice())
print(AFTER)
