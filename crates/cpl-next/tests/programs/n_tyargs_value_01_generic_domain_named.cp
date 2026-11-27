// expect: ok, output "6 6 2 3 YES 0 3"
// rule: R04.2.27
// rule: R04.2.20
// A generic library domain named with its arguments is a value as its name alone is: `for` walks
// `Pair<V>`, its walk's members (`count(p)`, `find`) answer on it, and a self-less `@get` fn
// reads without `()`.
struct Two<B>(i: Int):
    @get fn none() -> Two<B> = Two(0)
impl Index for Two<B>:
    let count = 2 * B.count
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> Two<B> = Two(k)
domain V = 0..3
domain W = 0..4
var n = 0
for p in Pair<W>: n += 1
let odd = Two<V>.count(t => t.i % 2 == 1)
let hit = if Two<V>.find(t => t.i == 4) == None: "NO" else: "YES"
print(n, Pair<W>.count, Pair<V>.count(p => p.lo.ord == 0), odd, hit, Two<V>.none.i, Two<W>.nth(3).i)
