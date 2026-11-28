// expect: ok, output "3\n-1 9\n20 3 -1\n4 0"
// `V(j)` is the pattern form of `V.at(x)` (R04.2.14): it matches an `Int` inside `V` and binds the
// position.
domain V = 1..=5
var s = 0
for i in 0..=12:
    if let V(j) = i - 10: s += Int(j)
print(s)
domain W = 0..10
fn at(i: Int) -> Int:
    if let W(j) = i - 10: return Int(j)
    -1
print(at(20), at(19))
fn where_(x: Int) -> Int =
    match x:
        V(j) if Int(j) % 2 == 0 => Int(j) * 10
        V(j) => Int(j)
        _ => -1
print(where_(2), where_(3), where_(7))
fn first(x: Int) -> Int:
    let V(k) = x else: return 0
    Int(k)
print(first(4), first(6))
