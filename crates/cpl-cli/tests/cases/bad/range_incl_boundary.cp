// error: closed range of cuts, which names no piece
let n: 2..=9 = read()
domain I = 1..=n
let a: [I]Int = read()
for i in I: print(sum(a[..=i.r]))
