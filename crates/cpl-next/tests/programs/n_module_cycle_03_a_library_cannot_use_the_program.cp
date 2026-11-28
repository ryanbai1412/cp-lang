// expect: E0206
// rule: R11.2.2
// A program is a leaf of the library graph: its inline module and a library naming each other
// are a cycle.
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/a.cp "use me.{one}\npub fn two() -> Int = one() + 1\n"
mod me:
    use kit.a.{two} // <- here
    pub fn one() -> Int = 1
    pub fn three() -> Int = two() + 1
use me.{three}
print(three())
