// expect: E0209
// rule: R11.2.2 R01.6.1
// Modules of one library may name each other, but a `const` whose value reaches itself through
// them has none: `kit.a.A` reads `kit.b.B`, which reads `A`.
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/a.cp "use kit.b.{B}\npub const A: Int = B + 1\n"
// file: kit/b.cp "use kit.a.{A}\npub const B: Int = A + 1\n"
use kit.a.{A}
print(A)
