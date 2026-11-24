// expect: E0503 at 2
// rule: 06.4.3
// tool: check
// has: "@alt(f)"
fn ways(n: Int) -> Int = n
@brute(ways)
fn slow(n: Int) -> Int = n
let n: 1..=8 = read()
print(ways(n))
