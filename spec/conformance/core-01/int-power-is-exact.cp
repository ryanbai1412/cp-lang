// expect: ok, output "1024 1 536870912"
// input: "536870912\n"
// rule: 01.5.8
// `Int ** Int` is the exact power, and a literal power folds into a static bound.
let x = read<1..=2 ** 29>()
print(2 ** 10, 3 ** 0, x)
