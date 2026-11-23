@!multitest
let n: 1..=20 = read()
@memo(into: ways)
fn f(i: 0..=n) -> Int:
    if i <= 1: return 1
    f(i - 1) + f(i - 2)
print(f(n), count(ways[i] != None for i in 0..=n))
// The table is the program's own: an answer put in ahead of a call is used.
ways[n] = 7
print(f(n))
