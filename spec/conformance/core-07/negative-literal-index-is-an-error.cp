// expect: E0321
// rule: 07.2.17
// `a[-1]` is a Python habit: keys never wrap, so a negative literal key is an error whose fix is `a.last!` (decision 508).
let a = [3, 1, 4]
print(a[-1]) // <- here
