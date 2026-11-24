// expect: E0503
// rule: 06.4.3
// tool: check
// has: "@alt(f)"
fn ways(n: Int) -> Int = n
@brute(ways) // <- here
fn slow(n: Int) -> Int = n
let n: 1..=8 = read()
print(ways(n))
