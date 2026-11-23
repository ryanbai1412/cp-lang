// expect: ok, output "5"
// rule: 10.2.3
// input: "2 4\n"
// A read's range bound may be any `Int` expression; `read<l + 1..=9>()` is a type argument, not a comparison (rebuild round 21 A5).
let l = read<1..=5>()
let r = read<l + 1..=9>()
print(r + 1)
