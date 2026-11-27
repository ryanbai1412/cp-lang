// expect: ok, output "YES NO 11 2 6"
// rule: R04.3.4
// A constant range an array is over in a signature or a field is one domain, wherever it's written.
fn has(self: [0..2]Int, i: Int, x: Int) -> Bool = self[i] == x
fn sum3(a: [1..=3]Int) -> Int:
    var s = 0
    for x in a: s += x
    s
struct Pair:
    cells: [0..2]Int
let a: [0..2]Int = [5, 6]
let p = Pair([4, 7])
var b: [1..=3]Int = 0
b[1] = 1
b[3] = 5
print(yes(a.has(0, 5)), yes(a.has(1, 5)), p.cells[0] + p.cells[1], p.cells.len, sum3(b))
