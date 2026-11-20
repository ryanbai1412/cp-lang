// error: `i + 1` is a ?I: it may step outside I
let n: 2..=9 = read()
domain I = 1..=n
let a: [I]Int = read()
for i in I: print(a[i + 1])
