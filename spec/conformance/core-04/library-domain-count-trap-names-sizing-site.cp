// expect: R0401 at 5
// input: "30\n"
// rule: 09.4.5
// tool: run
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub struct P<A>(i: Int)\nimpl Index for P<A>:\n    let count = sized(A.count)\n    @get fn ord(self) -> Int = i\n    fn nth(k: Int) -> P<A> = P(k)\nfn sized(n: Int) -> Int:\n    with checks(asserts: on):\n        assert n <= 25, \"too big\"\n    n\n"
// has: "kit/m.cp:8: inside `sized`"
// R09.4.5: an assert in a library domain's `count` traps at the user's line that sizes it.
use kit.m.{P}
let n: Int = read()
domain V = 0..n
var dp: [P<V>]Int = 0
print(len(dp))
