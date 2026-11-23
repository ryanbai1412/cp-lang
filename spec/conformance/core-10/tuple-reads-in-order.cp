// expect: ok, output "6"
// rule: 10.2.3
// input: "1 2 3\n"
// A tuple reads its fields in order (rebuild round 6 H-TUPLE-READ).
let (a, b, c) = read<(Int, Int, Int)>()
print(a + b + c)
