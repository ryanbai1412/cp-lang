// expect: ok, output "in solve\n8\n8"
// rule: 04.2.26
// input: "3\n"
// A branch that uses `Subset<Letter>` sizes it there, and a fn that uses it
// sizes it on its first call.
let m: 1..=26 = read()
domain Letter = 0..m
fn solve() -> Int:
    print("in solve")
    var latest: [Subset<Letter>]Int = 0
    latest.len
if m < 22:
    print(solve())
    var t: [Subset<Letter>]Int = 1
    print(t.sum)
else: print("NO")
