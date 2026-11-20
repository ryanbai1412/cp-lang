// cp: 0.1 8ef64b9+dirty
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var pre: [V.bounds]Int = 0
let x: Int = read()
@solution
fn main():
    if x in V:
        print(a[x])
