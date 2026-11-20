let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var pre: [V.bounds]Int = 0
@solution
fn main():
    let x = a[V.first] * a[V.last] * a[V.first]
    print(x)
