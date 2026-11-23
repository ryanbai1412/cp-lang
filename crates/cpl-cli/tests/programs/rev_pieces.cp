// The pieces of a `.rev` are cut in the order it walks, in both engines:
// `take`/`drop`/`take_last`/`drop_last` and `nth` count from the end the
// view starts at, keep the source's keys, and work on a name bound to a
// `.rev` (R07.3.5, R07.4.7).
let a = [10, 20, 30, 40]
let r = a.rev
print(List(a.rev.take(3)), List(a.rev.drop(1)))
print(List(r.take_last(1)), List(r.drop_last(3)))
print(List(r.drop(1).keys), r.take(2).first!, List(r.take(9)))
print(a.rev.nth(0), r.nth(3), r.drop(1).nth(0))
var s = 0
for x in r.take(2): s = s * 100 + x
print(s)
