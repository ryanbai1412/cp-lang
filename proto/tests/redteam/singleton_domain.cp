// the same boundaries when the domain has exactly one value: first == last
let n: 1..=5 = read()
domain V = 1..=n
var a: [V]Int = read()
let f = V.first
let z = V.last
print(sum(a[f.l..z.r]), sum(a[f.l..f.l]), sum(a[z.r..z.r]))
print(sum(a[z.l(n)..z.r]), sum(a[f.l..f.r(n)]), sum(a[z.l(0)..z.r]))
print(sum(a[f..=z]), a[f..=z].len, a[f.l..f.r].len)
// a range written backwards is empty, like an inverted domain
let back = a[z..=f]
print(back.len, sum(back))
