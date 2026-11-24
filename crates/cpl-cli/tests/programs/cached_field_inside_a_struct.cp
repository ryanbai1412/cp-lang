// A closure reading a struct that holds a Dsu may halve its paths.
struct Holder(pub d: Dsu<V>)
domain V = 0..3
let h = Holder(Dsu<V>(rollback: true))
let ok = (x: Holder) => x.d.components >= 2
print(yes(ok(h)))
