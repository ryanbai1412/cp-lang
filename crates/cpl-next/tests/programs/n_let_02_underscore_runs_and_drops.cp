// expect: ok, output "2\n1 4\n7"
// input: "3 9 4\n"
// `let _ = e` runs `e` for its effects and keeps nothing (R06.1, R10): here it skips input
// tokens and drops a list it builds.
var calls = 0
fn bump(n: Int) -> List<Int>:
    calls += 1
    [n]
let _ = bump(1)
let _: List<Int> = bump(2)
print(calls)
let _ = read<Int>()
let a: Int = read()
let b: Int = read()
print(calls - 1, b)
let keep = bump(7)
print(keep)
