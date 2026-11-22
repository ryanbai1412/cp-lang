// cp: 0.1 a3da8be+dirty
// prefix sums with typed domains
let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
let qs: [q](l: V, r: V) = read() where all(x.l <= x.r for x in qs)

var pre: [V.cuts]Int = 0
for i in V:
    pre[i.after] = pre[i.before] + a[i]

@solution
fn main():
    for x in qs:
        print(pre[x.r.after] - pre[x.l.before])
