// expect: ok, output "4 9"
// rule: 02.1.4
fn first(xs: List<Int>) -> Int = xs[0]
let a = [4, 5]
let rows = [[7, 8], [9]]
let r = rows[1]
print(first(a), first(r))
