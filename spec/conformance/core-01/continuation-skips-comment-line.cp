// expect: ok, output "7\n5"
// rule: 01.2.4
// A blank or comment-only line is not a logical line, so the line after it still continues the expression above.
let a = 10
let b = a
    // take the cost off
    - 3
let c = [4, 9]
    .min()

    ?? 0
print(b)
print(c + 1)
