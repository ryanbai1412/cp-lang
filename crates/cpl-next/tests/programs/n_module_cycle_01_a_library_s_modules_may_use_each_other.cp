// expect: ok, output "3 4\n"
// rule: R11.2.2
// Modules of one library may name each other: `kit.a` uses `kit.b` and `kit.b` uses `kit.a`.
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/a.cp "use kit.b.{inc}\npub fn three() -> Int = inc(2)\npub fn two() -> Int = 2\n"
// file: kit/b.cp "use kit.a.{two}\npub fn inc(x: Int) -> Int = x + 1\npub fn four() -> Int = two() + two()\n"
use kit.a.{three}
use kit.b.{four}
print(three(), four())
