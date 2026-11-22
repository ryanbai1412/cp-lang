// expect: ok, output "YES NO YES\nYES NO\n7 12"
// rule: 06.1.8
domain V = 0..2
fn add(a: Int, b: Int = 5) -> Int = a + b
let vis = [true, false, true]
print(vis.map(yes))
let seen: [V]Bool = [true, false]
print(seen.map(yes))
print([2, 7].map(add))
