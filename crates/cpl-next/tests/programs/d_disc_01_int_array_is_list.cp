// expect: ok, output "3 1\n3 1 2\nYES 2"
// rule: R04.8.11 R07.3.5
// `[Int]T` is `List<T>`: a list's keys are `Int`, which has no count, so its length is its own.
fn second(x: [Int]Int) -> Int = x[1]
let a: [Int]Int = [3, 1, 2]
print(a.len, second(a))
print(a)
let b: List<Int> = a
print(yes(b == [3, 1, 2]), b.keys.len - 1)
