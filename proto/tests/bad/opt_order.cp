let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var pre: [V.cuts]Int = 0
var best: ?Int = none
@solution
fn main():
    for i in V:
        if best < a[i]:
            best = a[i]
