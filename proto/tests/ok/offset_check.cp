// cp: 0.1 329fa7d+dirty
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var pre: [V.bounds]Int = 0
@solution
fn main():
    for i in V:
        print(a[i + 1])
