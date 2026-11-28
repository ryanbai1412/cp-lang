// expect: E0211
// rule: 01.6.8
// engines: next
// Twenty million jumps back to a loop's start are past a `const`'s 10000000 steps; `let`
// would compute the same value at run time.
fn tri(n: Int) -> Int:
    var s = 0
    for i in 1..=n:
        s += i
    s
const T = tri(20000000) // <- here
print(T)
