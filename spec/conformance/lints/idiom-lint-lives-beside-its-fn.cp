// expect: hand_rolled_mid
// rule: 06.4.10
// An idiom whose fix is one library call is declared beside the fn it teaches, as `@lint`, not in the linter (decision 512).
/// The midpoint of `l` and `r`, rounded down.
@lint(hand_rolled_mid, from: "l + (r - l) / 2", to: "mid(l, r)")
pub fn mid(l: Int, r: Int) -> Int:
    l + (r - l) / 2
let (a, b): (Int, Int) = read()
print(a + (b - a) / 2) // <- here
