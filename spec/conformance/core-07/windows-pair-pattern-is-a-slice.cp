// expect: E0302
// rule: 07.7.10
// has: "each piece is a slice"
// The rule is the yielded slice's, not `runs`'s: a window can't be taken apart either.
let xs = [1, 2, 3]
for (a, b) in xs.windows(2): print(a, b) // <- here
