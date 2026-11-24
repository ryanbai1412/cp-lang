// Codeforces 2152 A (proto/ports/ecnerwala/cf2152a.cp), with a planted
// multitest leak: the set of values seen lives above the `@!multitest`
// divider, so a value an earlier test had is not counted again.
// class: multitest-leak
var seen: Set<Int> = Set()  // planted
fn operations(a: List<Int>) -> Int:
    var distinct = 0
    for x in a:
        if x not in seen:
            seen.insert(x)
            distinct += 1
    2 * distinct - 1

@alt(operations)
fn operations_slow(a: List<Int>) -> Int = 2 * Set(a).len - 1

@!multitest
let n: 1..=100 = read()
let a: [0..n]1..=n = read()
print(operations([x for x in a]))
