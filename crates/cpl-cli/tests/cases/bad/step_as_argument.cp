// error: expected I, got ?I: `i.next` may step outside I
let n: 2..=9 = read()
domain I = 1..=n
fn show(i: I): print(i)
for i in I: show(i.next)
