let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var pre: [V.cuts]Int = 0
@solution
fn main():
    for i in 1..=n:
        print(a[i])
