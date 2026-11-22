// error: range between I and J
let n: 2..=9 = read()
domain I = 1..=n
domain J = 1..=n
let a: [I]Int = read()
print(sum(a[I.first!..J.last!]))
