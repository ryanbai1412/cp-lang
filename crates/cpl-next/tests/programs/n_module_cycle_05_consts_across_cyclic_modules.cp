// expect: ok, output "3 4\n"
// rule: R11.2.2 R01.6.1
// A library's consts are set in the order their initializers read them, not in module order:
// `kit.a` and `kit.b` use each other, `A` reads `B`, and `C` reads `A`.
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/a.cp "use kit.b.{B}\npub const A: Int = B + 1\n"
// file: kit/b.cp "use kit.a.{A}\npub const B: Int = 2\npub const C: Int = A + 1\n"
use kit.a.{A}
use kit.b.{C}
print(A, C)
