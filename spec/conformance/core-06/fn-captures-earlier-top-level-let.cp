// expect: ok, output "4"
// rule: 06.1.7
// input: "3\n"
// A `fn` between top-level statements captures an earlier `let` (rebuild round 6 H-FN-PLACEMENT).
let n = read<Int>()
fn f(x: Int) -> Int = x + n
print(f(1))
