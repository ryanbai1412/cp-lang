// expect: ok, output "YES NO"
// rule: 01.2.4
// An indented line that starts with a binary operator continues the line above, `or` and `and` included (rebuild 8, 08-even-number-addicts).
fn same(a: Bool, b: Bool) -> Bool:
    (a and b)
        or (not a and not b)
let x = 3
let big = x > 2
    and x < 10
print(yes(same(true, true)), yes(same(true, false) and big))
