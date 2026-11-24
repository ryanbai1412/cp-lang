// expect: ok, output "YES NO"
// rule: 08.1.4
// A fn of a built-in member's name is the call when the built-in can't take the arguments: `contains(x)` takes one.
fn contains(self: [0..2]Int, i: Int, x: Int) -> Bool = self[i] == x
let a: [0..2]Int = [5, 6]
print(yes(a.contains(0, 5)), yes(a.contains(1, 5)))
