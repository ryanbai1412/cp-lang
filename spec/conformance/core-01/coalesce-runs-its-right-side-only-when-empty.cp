// expect: ok, output "1 0\n9 1\n4 2\n304 5"
// rule: 01.5.7
// `a ?? b` runs `b` only when `a` is `None`, and chains to the right, inside a loop as at the top level.
var calls = 0
fn side(x: ?Int) -> ?Int:
    calls += 1
    return x
let a: ?Int = Some(1)
let n: ?Int = None
print(a ?? side(Some(2)) ?? 9, calls)
print(n ?? side(None) ?? 9, calls)
print((n ?? side(Some(4)))!, calls)
var total = 0
for i in 0..5:
    let o: ?Int = if i % 2 == 0: None else: Some(i)
    total += o ?? side(Some(100))!
print(total, calls)
