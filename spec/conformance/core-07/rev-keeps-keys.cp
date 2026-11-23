// expect: ok, output "9 9 2"
// rule: 07.4.7
// `.rev` walks backwards and keeps keys: `nth(0)` is the first in walk order, and its key is the source's.
let b = [5, 9, 1].sorted().rev
print(b.nth(0), b.first!, b.keys.first!)
