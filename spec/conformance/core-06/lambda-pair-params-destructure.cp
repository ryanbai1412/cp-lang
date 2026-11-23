// expect: ok, output "3"
// rule: 06.1.4
// A two-parameter lambda over a sequence of pairs takes each pair apart: `(i, x) =>` means `((i, x)) =>` (rebuild 7, H-LAMBDA-TUPLE-PARAMS).
let xs = [3, 4, 5]
print(xs.items.count((i, x) => x == i + 3))
