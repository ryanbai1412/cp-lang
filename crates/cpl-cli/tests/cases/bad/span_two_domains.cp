// error: expected `I.cuts`, found `J.cuts`
let n: 2..=9 = read()
domain I = 1..=n
domain J = 1..=n
print(span(I.first!.l, J.last!.r).len)
