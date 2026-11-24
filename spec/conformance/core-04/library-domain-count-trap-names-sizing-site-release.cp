// expect: exit 3
// gap: compiler 2026-11-24T05Z
// input: "30\n"
// rule: 09.4.5
// tool: run --release
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub struct P<A>(i: Int)\nimpl Index for P<A>:\n    let count = sized(A.count)\n    @get fn ord(self) -> Int = i\n    fn nth(k: Int) -> P<A> = P(k)\nfn sized(n: Int) -> Int:\n    with checks(asserts: on):\n        assert n <= 25, \"too big\"\n    n\n"
// has: "n <= 25 (line 13): too big"
// has: "note: kit/m.cp:8: inside `sized`"
// R09.4.5: the C++ names the same sizing line as the interpreter, with the library's line as a note.
use kit.m.{P}
let n: Int = read()
domain V = 0..n
var dp: [P<V>]Int = 0
print(len(dp))
