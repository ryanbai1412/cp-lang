// expect: ok, output "3"
// rule: 08.1.4
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub struct T(k: Int):\n    pub fn foo(self) -> Int = self.k\npub fn foo(self: T) -> Int = 0 - self.k\n"
// lacks: "E0701"
// The E0701 for a fn a member hides belongs to the module that declares it: a file importing `T` calls the member.
use kit.m.{T}
print(T(3).foo())
