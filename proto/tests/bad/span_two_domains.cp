// error: span of a I and a J
let n: 2..=9 = read()
domain I = 1..=n
domain J = 1..=n
print(span(I.first, J.last).len)
