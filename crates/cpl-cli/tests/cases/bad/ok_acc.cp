// cp: 0.1 8ef64b9+dirty
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
@solution
fn main():
    var s = 0
    for i in V:
        s += a[i]
    print(s)
