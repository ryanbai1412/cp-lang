// expect: E0608
// rule: 06.4.3
// A `@check` fn is ordinary code under the ownership rules: a list moved in one round of a loop is moved for the next, as in the solution.
let n: 1..=5 = read()
@check
fn c() -> Bool:
    let a = [1, 2]
    for i in 0..3:
        var b = a // <- here
        b.push(i)
    true
print(n)
