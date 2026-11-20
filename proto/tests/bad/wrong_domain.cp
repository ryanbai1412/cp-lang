let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var pre: [V.bounds]Int = 0
domain E = 1..=n
let b: [E]Int = 0
@solution
fn main():
    for e in E:
        print(a[e])
