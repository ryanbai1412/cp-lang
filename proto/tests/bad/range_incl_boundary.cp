// error: `..=` ends at a position; a range up to the boundary `i.r` is written `..`
let n: 2..=9 = read()
domain I = 1..=n
let a: [I]Int = read()
for i in I: print(sum(a[..=i.r]))
