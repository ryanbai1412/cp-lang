// expect: ok, output "5 3 b 9"
// rule: 07.4.2
// `last` reads `iter_rev` through a `Bidir` bound: List, Range, Str and `[D]T` implement it in std.
fn lst<S: Bidir<T>, T>(s: S) -> ?T = s.last
domain D = 0..3
let a: [D]Int = [7, 8, 9]
print(lst([4, 5])!, lst(0..4)!, lst("ab")!, lst(a)!)
