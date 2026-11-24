// expect: ok, output "2 2 a -1"
// rule: 07.9.5
// `rfind` walks from the end through a `Bidir` bound on a List, a Range, a Str and a `[D]T`.
fn rf<S: Bidir<T>, T>(s: S, p: T -> Bool) -> ?T = s.rfind(p)
domain D = 0..3
let a: [D]Int = [7, 8, 9]
print(rf([1, 2, 3], x => x < 3)!, rf(0..9, x => x < 3)!, rf("abc", c => c < 'b')!, rf(a, x => x < 3) ?? -1)
