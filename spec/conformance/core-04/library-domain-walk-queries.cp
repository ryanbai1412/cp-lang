// expect: ok, output "3\n4\n5\n7 6 5 4 3 2 1 0\n0 2 4 6 8 10 12 14\n7 6 5 4\n28\nYES YES"
// rule: 04.2.20
// R04.2.20: a library domain is the Seq of its walk; find, count(p), min(by:), rev, comprehensions and for run over the walk.
struct Sub<B>(bits: Int)
impl Index for Sub<B>:
    let count = 1 << B.count
    @get fn ord(self) -> Int = bits
    fn nth(k: Int) -> Sub<B> = Sub(k)
domain V = 0..3
print(Sub<V>.find(s => s.bits > 2).map(s => s.bits) ?? -1)
print(Sub<V>.count(s => s.bits % 2 == 1))
print(Sub<V>.min(by: s => (s.bits - 5) * (s.bits - 5))!.bits)
print([s.bits for s in Sub<V>.rev])
print([s.bits * 2 for s in Sub<V>])
print([s.bits for s in Sub<V>.rev if s.bits > 3])
var t = 0
for s in Sub<V>: t += s.bits
print(t)
print(yes(Sub<V>.any(s => s.bits == 6)), yes(Sub<V>.all(s => s.bits < 8)))
