// expect: E0602 at 5
// rule: 05.10.3
domain V = 1..=4
var a = [V] v => 0
fn clobber(): a[V.first!] = 0
let part = a[V(2)..]
clobber()
print(part.sum())
