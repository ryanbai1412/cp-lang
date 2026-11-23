let n: 1..=3000 = read()
domain I = 0..n
let a: [I]Int = read()
var lead: [I.ranges]Int = 0
for r in I.ranges:
    let (l, last) = (r.first!, r.last!)
    lead[r] = if r.len == 1: a[l] else: max(a[l] - lead[r.drop(1)], a[last] - lead[r.drop_last(1)])
print(lead[I.range])
