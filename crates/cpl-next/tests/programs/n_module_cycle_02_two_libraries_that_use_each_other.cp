// expect: E0206
// rule: R11.2.2
// Libraries form an acyclic graph: `kit` uses `box` and `box` uses `kit`.
// file: cp.toml "[libs]\nkit = \"kit\"\nbox = \"box\"\n"
// file: kit/a.cp "use box.b.{two}\npub fn three() -> Int = two() + 1\npub fn one() -> Int = 1\n"
// file: box/b.cp "use kit.a.{one}\npub fn two() -> Int = one() + 1\n"
use kit.a.{three} // <- here
print(three())
