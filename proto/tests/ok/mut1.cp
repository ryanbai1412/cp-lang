// cp: 0.1 7a57971+dirty
let n: 1..=9 = read()
domain I = 1..=n
fn bump(mut a: [I]Int, i: I):
    a[i] += 10
var xs: [I]Int = 0
bump(mut xs, I.first)
print(xs)
