// expect: ok, output "12 5\n3"
// rule: R11.2.1 R11.3.1
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "use kit.n.{half}\npub struct Box(v: Int):\n    pub fn grown(self, by: Int) -> Box = Box(self.v + half(by))\npub fn twice(x: Int) -> Int = x * 2\n"
// file: kit/n.cp "pub fn half(x: Int) -> Int = x / 2\n"
// A library root `cp.toml` names under `[libs]` is loaded as std is: its modules by `use`,
// each other's too, and without the prelude.
use kit.m.{Box, twice}
let b = Box(3).grown(4)
print(twice(6), b.v)
print(Box(1).grown(5).v)
