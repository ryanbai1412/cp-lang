// expect: ok, output "9 9 2"
// rule: 07.4.7
// `.rev` walks backwards and keeps keys: `nth(0)` is the first in walk order, and its key is the source's. The source is named, since a view of a temporary can't be bound (R05.7.8).
let s = [5, 9, 1].sorted()
let b = s.rev
print(b.nth(0), b.first!, b.keys.first!)
