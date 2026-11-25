// expect: ok, output "1\n-1\n20\n2\n9 2\n7 1\n5 0\n100"
// `if let` and `while let` bind the value for the guarded code only (R02.4.5), chained
// with `and`, as a statement and as an expression.
fn find(xs: List<Int>, k: Int) -> ?Int:
    for i in 0..xs.len:
        if xs.get(i) == k: return i
    None
var xs: List<Int> = List.new()
xs.push(5)
xs.push(7)
xs.push(9)
if let i = find(xs, 7): print(i)
else: print(-1)
if let i = find(xs, 8): print(i)
else: print(-1)
if let i = find(xs, 9) and i > 1: print(i * 10)
if let i = find(xs, 5) and let j = find(xs, 9): print(i + j)
var k = 9
while let i = find(xs, k):
    print(k, i)
    k -= 2
let t = if let i = find(xs, 5): i + 100 else: 0
print(t)
