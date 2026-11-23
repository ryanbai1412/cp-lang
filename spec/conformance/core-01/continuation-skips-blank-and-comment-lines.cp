// expect: ok, output "7 1"
// rule: 01.2.4
// Blank and comment-only lines between a line and its leading-operator continuation are
// skipped, as they are inside a block.
let a = 10
let xs = [3, 1, 2]
let b = a
    // note
    - 3
let c = xs
    .min()

    ?? 0
print(b, c)
