// expect: ok, output "5"
// rule: 10.2.9
// input: "3\n7 8\n5\n"
// `let _ = read<..>()` consumes tokens the program doesn't need (round 22, H15).
let n: 1..=10 = read()
let _ = read<[n - 1]Int>()
let x: Int = read()
print(x)
