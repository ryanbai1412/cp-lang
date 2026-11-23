// expect: ok, output "8 0 7 2\n28\n3 7"
// rule: 04.2.27
// R04.2.27: a generic struct's instance `Sub<V>` has the domain members and walk of a library domain.
struct Sub<B>(bits: Int)
impl Index for Sub<B>:
    let count = 1 << B.count
    @get fn ord(self) -> Int = bits
    fn nth(k: Int) -> Sub<B> = Sub(k)
domain V = 0..3
print(Sub<V>.count, Sub<V>.first!.bits, Sub<V>.last!.bits, Sub<V>.nth(2).ord)
var t = 0
for s in Sub<V>: t += s.bits
print(t)
print(Sub<V>.count(s => s.bits > 4), Sub<V>.rev.first.map(s => s.bits) ?? -1)
