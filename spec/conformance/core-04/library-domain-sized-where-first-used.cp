// expect: ok, output "NO"
// rule: 04.2.26
// input: "26\n"
// `Subset<Letter>` is sized where it is first used: `solve` is never
// called, so the count over 26 values never runs.
let m: 1..=26 = read()
domain Letter = 0..m
fn solve() -> Bool:
    print("in solve")
    var latest: [Subset<Letter>]Int = 0
    latest.len > 0
if m < 22: print(yes(solve()))
else: print("NO")
