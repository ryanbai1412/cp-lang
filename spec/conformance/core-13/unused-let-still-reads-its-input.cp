// expect: ok, output "2"
// rule: 13.1.1
// input: "1 2\n"
// A `let` whose value is never used still runs where it is written, so it still consumes its token (adversarial round 46).
let unused = read<Int>()
let y = read<Int>()
print(y)
