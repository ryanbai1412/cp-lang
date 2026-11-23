// expect: ok, output "1 9 9"
// rule: 07.4.7
// `.rev` walks backwards but keeps keys: `b[0]` is the element at key 0; `b.nth(0)` is the first in walk order.
let b = [5, 9, 1].sorted().rev
print(b[0], b.nth(0), b.first!)
