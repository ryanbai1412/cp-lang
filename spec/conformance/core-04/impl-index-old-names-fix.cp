// expect: ok
// rule: 04.2.27
// tool: fix
// rewritten: "    let count = 2\n    @get fn ord(self) -> Int = Int(b)\n    fn nth(i: Int) -> Two = Two(i == 1)"
// rewritten: "print(Two.count, Two.nth(1).ord)"
// `cpl fix` moves an `impl Index` and its uses from `K`/`to_idx`/`from_idx` to `count`/`ord`/`nth` (decision 504).
struct Two:
    b: Bool
impl Index for Two:
    let K = 2
    fn to_idx(self) -> Int = Int(b)
    fn from_idx(i: Int) -> Two = Two(i == 1)
print(Two.K, Two.from_idx(1).to_idx)
